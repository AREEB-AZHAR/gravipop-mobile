package com.gravipop.celestialmerge

import android.app.Activity
import android.util.Log
import com.google.android.gms.ads.AdRequest
import com.google.android.gms.ads.LoadAdError
import com.google.android.gms.ads.interstitial.InterstitialAd
import com.google.android.gms.ads.interstitial.InterstitialAdLoadCallback
import com.google.android.gms.ads.rewarded.RewardedAd
import com.google.android.gms.ads.rewarded.RewardedAdLoadCallback

class AdMobHelper(private val activity: Activity) {
    private val TAG = "GraviPop_AdMob"

    // Google AdMob official test ad units
    private val REWARDED_TEST_UNIT_ID = "ca-app-pub-3940256099942544/5224354917"
    private val INTERSTITIAL_TEST_UNIT_ID = "ca-app-pub-3940256099942544/1033173712"

    private var rewardedAd: RewardedAd? = null
    private var interstitialAd: InterstitialAd? = null

    fun loadRewardedAd() {
        val adRequest = AdRequest.Builder().build()
        RewardedAd.load(activity, REWARDED_TEST_UNIT_ID, adRequest, object : RewardedAdLoadCallback() {
            override fun onAdFailedToLoad(adError: LoadAdError) {
                Log.w(TAG, "Rewarded ad failed to load: ${adError.message}")
                rewardedAd = null
            }
            override fun onAdLoaded(ad: RewardedAd) {
                Log.d(TAG, "Rewarded ad loaded successfully.")
                rewardedAd = ad
            }
        })
    }

    fun showRewardedAd(onRewardEarned: (rewardType: Int) -> Unit) {
        activity.runOnUiThread {
            rewardedAd?.let { ad ->
                ad.show(activity) { rewardItem ->
                    Log.d(TAG, "User earned reward: ${rewardItem.amount} ${rewardItem.type}")
                    onRewardEarned(1)
                }
                rewardedAd = null
                loadRewardedAd() // Pre-load next
            } ?: run {
                Log.w(TAG, "Rewarded ad was not ready.")
                loadRewardedAd()
            }
        }
    }

    fun loadInterstitialAd() {
        val adRequest = AdRequest.Builder().build()
        InterstitialAd.load(activity, INTERSTITIAL_TEST_UNIT_ID, adRequest, object : InterstitialAdLoadCallback() {
            override fun onAdFailedToLoad(adError: LoadAdError) {
                Log.w(TAG, "Interstitial failed to load: ${adError.message}")
                interstitialAd = null
            }
            override fun onAdLoaded(ad: InterstitialAd) {
                Log.d(TAG, "Interstitial loaded.")
                interstitialAd = ad
            }
        })
    }

    fun showInterstitialAd() {
        activity.runOnUiThread {
            interstitialAd?.show(activity)
            interstitialAd = null
            loadInterstitialAd()
        }
    }
}
