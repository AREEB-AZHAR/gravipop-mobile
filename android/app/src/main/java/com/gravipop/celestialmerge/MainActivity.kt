package com.gravipop.celestialmerge

import android.app.NativeActivity
import android.os.Bundle
import android.util.Log
import com.google.android.gms.ads.MobileAds

class MainActivity : NativeActivity() {
    private val TAG = "GraviPop_MainActivity"
    lateinit var adMobHelper: AdMobHelper
    lateinit var billingHelper: BillingHelper

    companion object {
        init {
            // Load the native shared library compiled from Rust
            try {
                System.loadLibrary("gravipop_mobile")
            } catch (e: UnsatisfiedLinkError) {
                Log.e("GraviPop", "Native library gravipop_mobile not found: ${e.message}")
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Initialize Google Mobile Ads SDK
        MobileAds.initialize(this) { initializationStatus ->
            Log.d(TAG, "AdMob Initialized: $initializationStatus")
        }

        adMobHelper = AdMobHelper(this)
        adMobHelper.loadRewardedAd()
        adMobHelper.loadInterstitialAd()

        billingHelper = BillingHelper(this)
        billingHelper.startConnection {
            Log.d(TAG, "Billing ready")
        }
    }

    // JNI Native Entry Points called from Rust
    fun showRewardedAdFromNative(rewardId: Int) {
        adMobHelper.showRewardedAd { rewardType ->
            onNativeAdRewardEarned(rewardType)
        }
    }

    fun showInterstitialAdFromNative() {
        adMobHelper.showInterstitialAd()
    }

    fun launchPurchaseFromNative(productId: String) {
        billingHelper.purchaseProduct(productId)
    }

    // Callbacks to Rust native code
    private external fun onNativeAdRewardEarned(rewardType: Int)
    private external fun onNativePurchaseCompleted(productId: String, token: String)
}
