// This class is a proxy between the Rust side P2PSession and the native WifiP2PManager.
// @see https://developer.android.com/reference/android/net/wifi/p2p/WifiP2pManager

package io.crisal.ngn;

import android.Manifest;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.net.MacAddress;
import android.net.wifi.p2p.WifiP2pConfig;
import android.net.wifi.p2p.WifiP2pDevice;
import android.net.wifi.p2p.WifiP2pDeviceList;
import android.net.wifi.p2p.WifiP2pGroup;
import android.net.wifi.p2p.WifiP2pInfo;
import android.net.wifi.p2p.WifiP2pManager;
import android.os.Build;
import android.os.Looper;
import android.util.Log;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.annotation.RequiresPermission;
import androidx.core.content.ContextCompat;

import java.util.Objects;


// An ActionListener implementation that resolves a native promise.
class ActionListenerWrapper implements WifiP2pManager.ActionListener {
    private static native long ngn_promise_resolve(long aNativePromise, int error);

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

public class NgnSessionProxy extends BroadcastReceiver implements WifiP2pManager.ChannelListener, WifiP2pManager.GroupInfoListener, WifiP2pManager.ConnectionInfoListener, WifiP2pManager.PeerListListener, WifiP2pManager.DeviceInfoListener {
    public static String TAG = "NgnSessionProxy";

    private static native long ngn_session_init(NgnSessionProxy session, String name);
    private static native long ngn_init();

    // BroadcastReceiver
    @Override
    public void onReceive(Context context, Intent intent) {
        final String action = intent.getAction();
        switch (Objects.requireNonNull(action)) {
            case WifiP2pManager.WIFI_P2P_STATE_CHANGED_ACTION: {
                // UI update to indicate wifi p2p status.
                final int state = intent.getIntExtra(WifiP2pManager.EXTRA_WIFI_STATE, -1);
                // Wifi Direct mode is enabled
                Toast.makeText(context, "Wifi direct state change: " + state, Toast.LENGTH_SHORT).show();
                break;
            }
            case WifiP2pManager.WIFI_P2P_PEERS_CHANGED_ACTION: {
                Toast.makeText(context, "P2P peers changed", Toast.LENGTH_SHORT).show();
                // TODO: Request available peers.
                break;
            }
            case WifiP2pManager.WIFI_P2P_CONNECTION_CHANGED_ACTION: {
                // TODO: Handle connect/disconnect?
                break;
            }
            case WifiP2pManager.WIFI_P2P_THIS_DEVICE_CHANGED_ACTION: {
                // val wifiP2pDevice = intent.getParcelableExtra(WifiP2pManager.EXTRA_WIFI_P2P_DEVICE, WifiP2pDevice::class.java)
                final WifiP2pDevice wifiP2pDevice = intent.getParcelableExtra(WifiP2pManager.EXTRA_WIFI_P2P_DEVICE);
                assert wifiP2pDevice != null;
                Toast.makeText(context, "P2P device changed: " + wifiP2pDevice, Toast.LENGTH_LONG).show();
                break;
            }
            default:
                Log.d(TAG, "Unknown action: " + action);
                break;
        }
    }

    // ChannelListener
    @Override
    public void onChannelDisconnected() {
        Log.d(TAG, "onChannelDisconnected");
        m_channel = null;
        // TODO: Notify the native object / maybe recreate the native session?
        initChannel();
    }

    // ConnectionInfoListener
    @Override
    public void onConnectionInfoAvailable(WifiP2pInfo info) {
        Log.d(TAG, "onConnectionInfoAvailable: go=" + info.isGroupOwner + ", goaddr=" + info.groupOwnerAddress);
    }

    // GroupInfoListener
    @Override
    public void onGroupInfoAvailable(WifiP2pGroup group) {
        Log.d(TAG, "onGroupInfoAvailable: " + group);
    }

    // PeerListListener
    @Override
    public void onPeersAvailable(WifiP2pDeviceList peers) {
        Log.d(TAG, "onPeersAvailable: " + peers);
    }

    // DeviceInfoListener
    @Override
    public void onDeviceInfoAvailable(@Nullable WifiP2pDevice wifiP2pDevice) {
        Log.d(TAG, "onDeviceInfoAvailable: " + wifiP2pDevice);
        if (wifiP2pDevice != null) {
            m_native = ngn_session_init(this, wifiP2pDevice.deviceName);
            Log.d(TAG, "onDeviceInfoAvailable got session: " + m_native);
        }
    }

    static {
        System.loadLibrary("ngn");
    }

    public NgnSessionProxy(Context aContext) {
        ngn_init();
        m_context = aContext;
        m_intentFilter = new IntentFilter();
        m_intentFilter.addAction(WifiP2pManager.WIFI_P2P_STATE_CHANGED_ACTION);
        m_intentFilter.addAction(WifiP2pManager.WIFI_P2P_PEERS_CHANGED_ACTION);
        m_intentFilter.addAction(WifiP2pManager.WIFI_P2P_CONNECTION_CHANGED_ACTION);
        m_intentFilter.addAction(WifiP2pManager.WIFI_P2P_THIS_DEVICE_CHANGED_ACTION);
    }

    /**
     * Initializes the P2P session. This needs to be outside the constructor so that the app can
     * make sure to obtain the right permissions.
     *
     * @param aP2pManager WifiP2pManager
     */
    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public void init() {
        m_manager = m_context.getSystemService(WifiP2pManager.class);
        assert m_manager != null;
        initChannel();
        onResume();
        m_manager.requestDeviceInfo(m_channel, this);
    }

    /**
     * Must be called when the `Context` passed to the constructor gets suspended.
     * Make sure to also call onResume().
     */
    public void onPause() {
        m_context.unregisterReceiver(this);
    }

    /**
     * Starts listening to system broadcasts.
     */
    public void onResume() {
        // NOTE: RECEIVER_EXPORTED as per [1]:
        //     Some system broadcasts come from highly privileged apps, such as Bluetooth and
        //     telephony, that are part of the Android framework but don't run under the system's
        //     unique process ID (UID). To receive all system broadcasts, including broadcasts from
        //     highly privileged apps, flag your receiver with RECEIVER_EXPORTED.
        // [1]: https://developer.android.com/develop/background-work/background-tasks/broadcasts#receiving-broadcasts
        ContextCompat.registerReceiver(m_context, this, m_intentFilter, ContextCompat.RECEIVER_EXPORTED);
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
    IntentFilter m_intentFilter;
    long m_native;
}
