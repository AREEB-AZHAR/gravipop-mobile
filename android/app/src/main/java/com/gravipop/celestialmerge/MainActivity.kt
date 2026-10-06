package com.gravipop.celestialmerge

import android.os.Bundle
import com.gravipop.runtime.QuadActivity

/** The Miniquad SurfaceView host runs Rust, and AdMob overlays that same host.
 * A NativeActivity cannot host Miniquad's Java/JNI runtime correctly.
 */
class MainActivity : QuadActivity() {
    private lateinit var ads: AdMobHelper

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        ads = AdMobHelper(this, ::onNativeAdAvailability, ::onNativeAdFinished)
        ads.start()
    }

    fun getSaveDirectoryFromNative(): String = filesDir.absolutePath

    // Called by the Rust render thread; SDK interactions belong to the UI thread.
    fun requestAdFromNative(requestId: Int, rewarded: Boolean): Boolean {
        if (!::ads.isInitialized || isFinishing || isDestroyed) return false
        runOnUiThread { ads.show(requestId, rewarded) }
        return true
    }

    fun showAdPrivacyOptionsFromNative(): Boolean {
        if (!::ads.isInitialized || isFinishing || isDestroyed) return false
        runOnUiThread { ads.showPrivacyOptions() }
        return true
    }

    override fun onDestroy() {
        if (::ads.isInitialized) ads.destroy()
        super.onDestroy()
    }

    private external fun onNativeAdAvailability(flags: Int)
    private external fun onNativeAdFinished(requestId: Int, result: Int)
}
