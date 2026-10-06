package com.gravipop.celestialmerge

import android.app.Activity
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import android.util.Log
import com.google.android.libraries.ads.mobile.sdk.MobileAds
import com.google.android.libraries.ads.mobile.sdk.common.AdLoadCallback
import com.google.android.libraries.ads.mobile.sdk.common.AdRequest
import com.google.android.libraries.ads.mobile.sdk.common.FullScreenContentError
import com.google.android.libraries.ads.mobile.sdk.common.LoadAdError
import com.google.android.libraries.ads.mobile.sdk.initialization.InitializationConfig
import com.google.android.libraries.ads.mobile.sdk.interstitial.InterstitialAd
import com.google.android.libraries.ads.mobile.sdk.interstitial.InterstitialAdEventCallback
import com.google.android.libraries.ads.mobile.sdk.rewarded.RewardedAd
import com.google.android.libraries.ads.mobile.sdk.rewarded.RewardedAdEventCallback
import com.google.android.ump.ConsentInformation
import com.google.android.ump.ConsentRequestParameters
import com.google.android.ump.UserMessagingPlatform

/** SDK callbacks are marshalled to the UI thread and sent to Rust with the
 * original request ID. A reward is delivered only after the rewarded ad closes.
 */
class AdMobHelper(
    private val activity: Activity,
    private val onAvailability: (Int) -> Unit,
    private val onFinished: (Int, Int) -> Unit,
) {
    private val handler = Handler(Looper.getMainLooper())
    private val consent = UserMessagingPlatform.getConsentInformation(activity)
    private var initialized = false
    private var initializing = false
    private var destroyed = false
    private var rewardedAd: RewardedAd? = null
    private var interstitialAd: InterstitialAd? = null
    private var rewardedLoading = false
    private var interstitialLoading = false
    private var rewardedLoadedAt = 0L
    private var interstitialLoadedAt = 0L
    private var activeRequest: Int? = null
    private var rewardEarned = false

    fun start() {
        if (BuildConfig.ADS_TEST_MODE) {
            initializeSdk() // Official sample IDs have no publisher consent message.
            return
        }
        consent.requestConsentInfoUpdate(activity, ConsentRequestParameters.Builder().build(), {
            UserMessagingPlatform.loadAndShowConsentFormIfRequired(activity) { error ->
                if (error != null) Log.w(TAG, "Consent form: ${error.message}")
                updateAvailability()
                if (consent.canRequestAds()) initializeSdk()
            }
        }, { error ->
            Log.w(TAG, "Consent update: ${error.message}")
            updateAvailability()
            if (consent.canRequestAds()) initializeSdk()
        })
        if (consent.canRequestAds()) initializeSdk()
    }

    private fun initializeSdk() {
        if (initialized || initializing || destroyed) return
        initializing = true
        Thread {
            try {
                MobileAds.initialize(activity, InitializationConfig.Builder(BuildConfig.ADMOB_APP_ID).build()) {
                    activity.runOnUiThread {
                        if (!destroyed) {
                            initializing = false
                            initialized = true
                            loadRewardedAd()
                            loadInterstitialAd()
                        }
                    }
                }
            } catch (error: Exception) {
                activity.runOnUiThread {
                    initializing = false
                    Log.e(TAG, "Ads initialization failed", error)
                    updateAvailability()
                }
            }
        }.start()
    }

    private fun canLoad(): Boolean = initialized && !destroyed &&
        (BuildConfig.ADS_TEST_MODE || consent.canRequestAds())

    private fun loadRewardedAd() {
        if (!canLoad() || rewardedLoading || rewardedAd != null) return
        rewardedLoading = true
        RewardedAd.load(AdRequest.Builder(BuildConfig.ADMOB_REWARDED_UNIT_ID).build(),
            object : AdLoadCallback<RewardedAd> {
                override fun onAdLoaded(ad: RewardedAd) = activity.runOnUiThread {
                    rewardedLoading = false
                    if (destroyed) return@runOnUiThread
                    rewardedAd = ad
                    rewardedLoadedAt = SystemClock.elapsedRealtime()
                    updateAvailability()
                }
                override fun onAdFailedToLoad(error: LoadAdError) = activity.runOnUiThread {
                    rewardedLoading = false
                    Log.w(TAG, "Rewarded ad unavailable: ${error.message}")
                    updateAvailability()
                    if (!destroyed) handler.postDelayed({ loadRewardedAd() }, 30_000)
                }
            })
    }

    private fun loadInterstitialAd() {
        if (!canLoad() || interstitialLoading || interstitialAd != null) return
        interstitialLoading = true
        InterstitialAd.load(AdRequest.Builder(BuildConfig.ADMOB_INTERSTITIAL_UNIT_ID).build(),
            object : AdLoadCallback<InterstitialAd> {
                override fun onAdLoaded(ad: InterstitialAd) = activity.runOnUiThread {
                    interstitialLoading = false
                    if (destroyed) return@runOnUiThread
                    interstitialAd = ad
                    interstitialLoadedAt = SystemClock.elapsedRealtime()
                    updateAvailability()
                }
                override fun onAdFailedToLoad(error: LoadAdError) = activity.runOnUiThread {
                    interstitialLoading = false
                    Log.w(TAG, "Interstitial unavailable: ${error.message}")
                    updateAvailability()
                    if (!destroyed) handler.postDelayed({ loadInterstitialAd() }, 30_000)
                }
            })
    }

    fun show(requestId: Int, rewarded: Boolean) {
        if (destroyed || activeRequest != null || !canLoad() || activity.isFinishing) {
            onFinished(requestId, 3)
            return
        }
        // Google ad objects expire after approximately an hour.
        val now = SystemClock.elapsedRealtime()
        if (now - rewardedLoadedAt > 55 * 60_000) rewardedAd = null
        if (now - interstitialLoadedAt > 55 * 60_000) interstitialAd = null
        if ((rewarded && rewardedAd == null) || (!rewarded && interstitialAd == null)) {
            onFinished(requestId, 3)
            loadRewardedAd()
            loadInterstitialAd()
            updateAvailability()
            return
        }
        activeRequest = requestId
        rewardEarned = false
        try {
            if (rewarded) {
                val ad = rewardedAd!!
                rewardedAd = null
                ad.adEventCallback = object : RewardedAdEventCallback {
                    override fun onAdDismissedFullScreenContent() = activity.runOnUiThread {
                        finish(requestId, if (rewardEarned) 1 else 2)
                    }
                    override fun onAdFailedToShowFullScreenContent(error: FullScreenContentError) = activity.runOnUiThread {
                        Log.w(TAG, "Rewarded ad could not show: ${error.message}")
                        finish(requestId, 3)
                    }
                }
                ad.show(activity) {
                    activity.runOnUiThread {
                        if (activeRequest == requestId) rewardEarned = true
                    }
                }
            } else {
                val ad = interstitialAd!!
                interstitialAd = null
                ad.adEventCallback = object : InterstitialAdEventCallback {
                    override fun onAdDismissedFullScreenContent() = activity.runOnUiThread { finish(requestId, 2) }
                    override fun onAdFailedToShowFullScreenContent(error: FullScreenContentError) = activity.runOnUiThread {
                        Log.w(TAG, "Interstitial could not show: ${error.message}")
                        finish(requestId, 3)
                    }
                }
                ad.show(activity)
            }
            updateAvailability()
        } catch (error: Exception) {
            Log.e(TAG, "Ad display failed", error)
            finish(requestId, 3)
        }
    }

    private fun finish(requestId: Int, result: Int) {
        if (activeRequest != requestId) return
        activeRequest = null
        rewardEarned = false
        onFinished(requestId, result)
        loadRewardedAd()
        loadInterstitialAd()
        updateAvailability()
    }

    fun showPrivacyOptions() {
        UserMessagingPlatform.showPrivacyOptionsForm(activity) { error ->
            if (error != null) Log.w(TAG, "Privacy options: ${error.message}")
            if (!consent.canRequestAds()) {
                rewardedAd = null
                interstitialAd = null
            } else if (!initialized) initializeSdk()
            else { loadRewardedAd(); loadInterstitialAd() }
            updateAvailability()
        }
    }

    private fun updateAvailability() {
        var flags = 0
        if (!destroyed && activeRequest == null && canLoad()) {
            if (rewardedAd != null) flags = flags or 1
            if (interstitialAd != null) flags = flags or 2
        }
        if (!BuildConfig.ADS_TEST_MODE && consent.privacyOptionsRequirementStatus ==
            ConsentInformation.PrivacyOptionsRequirementStatus.REQUIRED) flags = flags or 4
        onAvailability(flags)
    }

    fun destroy() {
        destroyed = true
        handler.removeCallbacksAndMessages(null)
        activeRequest?.let { onFinished(it, 3) }
        activeRequest = null
        rewardedAd = null
        interstitialAd = null
        onAvailability(0)
    }

    private companion object { const val TAG = "GraviPop_Ads" }
}
