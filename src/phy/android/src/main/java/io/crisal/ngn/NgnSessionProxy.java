// This class is a proxy between the Rust side P2PSession and the native WifiP2PManager.
// @see https://developer.android.com/reference/android/net/wifi/p2p/WifiP2pManager

package io.crisal.ngn;

import android.Manifest;
import android.content.Context;
import android.net.MacAddress;
import android.net.wifi.p2p.WifiP2pConfig;
import android.net.wifi.p2p.WifiP2pDevice;
import android.net.wifi.p2p.WifiP2pManager;
import android.os.Build;
import android.os.Looper;
import android.util.Log;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.annotation.RequiresPermission;

public class NgnSessionProxy implements WifiP2pManager.ChannelListener {
    public static String TAG = "NgnSessionProxy";
    private static native long ngn_session_init(NgnSessionProxy session, String deviceAddress);

    private static native long ngn_promise_resolve(long aNativePromise, int error);

    @Override
    public void onChannelDisconnected() {
        Log.d(TAG, "onChannelDisconnected");
        m_channel = null;
        // TODO: Notify the native object / maybe recreate the native session?
        initChannel();
    }

    // An ActionListener implementation that resolves a native promise.
    public class ActionListenerWrapper implements WifiP2pManager.ActionListener {
        ActionListenerWrapper(long aNativePromise) {
            this.m_native = aNativePromise;
        }

        long m_native;

        @Override
        public void onSuccess() {
            ngn_promise_resolve(m_native, 0);
            m_native = 0;
        }

        @Override
        public void onFailure(int reason) {
            ngn_promise_resolve(m_native, reason);
            m_native = 0;
        }
    }

    static {
        System.loadLibrary("ngn");
    }

    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public NgnSessionProxy(Context aContext, @NonNull WifiP2pManager aP2pManager) {
        m_context = aContext;
        m_manager = aP2pManager;
        initChannel();
        WifiP2pManager.DeviceInfoListener deviceInfoListener = new WifiP2pManager.DeviceInfoListener() {
            @Override
            public void onDeviceInfoAvailable(@Nullable WifiP2pDevice wifiP2pDevice) {
                Log.d(TAG, "Got mac address: " + wifiP2pDevice.deviceAddress.toString());
                m_native = ngn_session_init(NgnSessionProxy.this, wifiP2pDevice.deviceAddress.toString());
            }
        };
        m_manager.requestDeviceInfo(m_channel, deviceInfoListener);
    }

    private void initChannel() {
        m_channel = m_manager.initialize(m_context, Looper.getMainLooper(), this);
    }

    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    void discoverPeers(long aNativePromise) {
        m_manager.discoverPeers(m_channel, new ActionListenerWrapper(aNativePromise));
    }

    // TODO(emilio): Maybe use byte[] rather than string to pass around mac addresses.
    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    void connectToPeer(String aMacAddress, long aNativePromise) {
        final WifiP2pConfig.Builder builder = new WifiP2pConfig.Builder();
        builder.setDeviceAddress(MacAddress.fromString(aMacAddress));
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            builder.setGroupClientIpProvisioningMode(WifiP2pConfig.GROUP_CLIENT_IP_PROVISIONING_MODE_IPV6_LINK_LOCAL);
        } else {
            Log.w(TAG, "Client IP provisioning might not use IPv6 link-local addressing");
        }
        m_manager.connect(m_channel, builder.build(), new ActionListenerWrapper(aNativePromise));
    }

    Context m_context;
    WifiP2pManager m_manager;
    WifiP2pManager.Channel m_channel;
    long m_native;
}
