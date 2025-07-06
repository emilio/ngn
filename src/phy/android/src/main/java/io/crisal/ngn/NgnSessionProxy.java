// This class is a proxy between the Rust side P2PSession and the native WifiP2PManager.
// @see https://developer.android.com/reference/android/net/wifi/p2p/WifiP2pManager

package io.crisal.ngn;

import android.Manifest;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.net.MacAddress;
import android.net.NetworkInfo;
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

import androidx.annotation.Nullable;
import androidx.annotation.RequiresPermission;
import androidx.core.content.ContextCompat;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Objects;
import java.util.function.Function;

// An ActionListener implementation that resolves a native promise.
class ActionListenerNativeAdapter implements WifiP2pManager.ActionListener {
    private static native long ngn_promise_resolve(long aNativePromise, int error);

    ActionListenerNativeAdapter(long aNativePromise) {
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

// An ActionListener implementation that resolves a Function<>.
class ActionListenerFunctionAdapter implements WifiP2pManager.ActionListener {
    ActionListenerFunctionAdapter(Function<Boolean, Void> aFunction) {
        m_function = aFunction;
    }

    void resolve(boolean aResult) {
        if (m_function == null) {
            return;
        }
        try {
            m_function.apply(aResult);
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }

    @Override
    public void onSuccess() {
        resolve(true);
    }

    @Override
    public void onFailure(int reason) {
        resolve(false);
    }

    Function<Boolean, Void> m_function;
}

public class NgnSessionProxy extends BroadcastReceiver implements WifiP2pManager.ChannelListener, WifiP2pManager.GroupInfoListener, WifiP2pManager.ConnectionInfoListener, WifiP2pManager.PeerListListener, WifiP2pManager.DeviceInfoListener {
    public static String TAG = "NgnSessionProxy";

    private static native long ngn_session_init(NgnSessionProxy session, String name);

    private static native long ngn_session_update_peers(long native_session, String[] peer_details);

    private static native void ngn_session_drop(long native_session);

    private static native void ngn_session_group_lost(long native_session);
    private static native void ngn_session_group_joined(long native_session, boolean is_go, String go_device_address, String interface_name, String owner_ip_address);

    private static native void ngn_init();

    // BroadcastReceiver
    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    @Override
    public void onReceive(Context context, Intent intent) {
        final String action = intent.getAction();
        Log.d(TAG, "BroadcastReceiver.onReceive(" + action + ")");
        switch (Objects.requireNonNull(action)) {
            case WifiP2pManager.WIFI_P2P_STATE_CHANGED_ACTION: {
                // UI update to indicate wifi p2p status.
                final int state = intent.getIntExtra(WifiP2pManager.EXTRA_WIFI_STATE, -1);
                Log.d(TAG, "Wifi direct state change: " + state);
                Toast.makeText(context, "Wifi direct state change: " + state, Toast.LENGTH_SHORT).show();
                break;
            }
            case WifiP2pManager.WIFI_P2P_PEERS_CHANGED_ACTION: {
                Log.d(TAG, "Peer list changed");
                m_manager.requestPeers(m_channel, this);
                break;
            }
            case WifiP2pManager.WIFI_P2P_CONNECTION_CHANGED_ACTION: {
                final WifiP2pGroup group = intent.getParcelableExtra(WifiP2pManager.EXTRA_WIFI_P2P_GROUP);
                final NetworkInfo networkInfo = intent.getParcelableExtra(WifiP2pManager.EXTRA_NETWORK_INFO);
                assert networkInfo != null;

                Log.d(TAG, "Connection change: " + group);
                Log.d(TAG, " > NetworkInfo: " + networkInfo);
                ConnectionState state = ConnectionState.Disconnected;
                if (networkInfo.isConnectedOrConnecting()) {
                    m_manager.requestConnectionInfo(m_channel, this);
                    state = networkInfo.isConnected() ? ConnectionState.Connected : ConnectionState.Connecting;
                }
                m_listener.connectionStateChanged(state);
                break;
            }
            case WifiP2pManager.WIFI_P2P_THIS_DEVICE_CHANGED_ACTION: {
                final WifiP2pDevice wifiP2pDevice = intent.getParcelableExtra(WifiP2pManager.EXTRA_WIFI_P2P_DEVICE);
                assert wifiP2pDevice != null;
                Log.d(TAG, "DEVICE_CHANGED(" + wifiP2pDevice.deviceName + "): " + wifiP2pDevice);
                Toast.makeText(context, "P2P device changed: " + wifiP2pDevice.deviceName, Toast.LENGTH_LONG).show();
                m_manager.requestConnectionInfo(m_channel, this);
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
    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    @Override
    public void onConnectionInfoAvailable(WifiP2pInfo info) {
        Log.d(TAG, "onConnectionInfoAvailable: " + info);
        m_connectionInfo = info;
        if (info.groupFormed) {
            m_manager.requestGroupInfo(m_channel, this);
        }
    }

    // GroupInfoListener
    @Override
    public void onGroupInfoAvailable(WifiP2pGroup group) {
        Log.d(TAG, "onGroupInfoAvailable: " + group);
        if (m_native == 0) {
            return;
        }
        assert m_connectionInfo != null;
        if (group == null || !m_connectionInfo.groupFormed) {
            if (m_currentGroup == null) {
                return;
            }
            m_currentGroup = null;
            ngn_session_group_lost(m_native);
            return;
        }
        if (m_currentGroup != null) {
            return;
        }
        m_currentGroup = group;
        // TODO(emilio): Would be nice to have group.getOwner().getIpAddress(), but that of course is Android 15 only :'(
        // Also, it'd be very nice to be able to get the owner interface address, but that is not exposed: WifiP2pDevice
        // _does_ have it in the android source code, but can't call it from the outside...
        ngn_session_group_joined(m_native, m_connectionInfo.isGroupOwner, group.getInterface(), group.getOwner().deviceAddress, m_connectionInfo.groupOwnerAddress.getHostAddress());
    }

    // PeerListListener
    @Override
    public void onPeersAvailable(WifiP2pDeviceList peers) {
        Log.d(TAG, "onPeersAvailable: " + peers);
        Toast.makeText(m_context, "P2P peers changed: " + peers, Toast.LENGTH_LONG).show();
        m_peerList = peers;
        peerListChanged();
    }

    // DeviceInfoListener
    @Override
    public void onDeviceInfoAvailable(@Nullable WifiP2pDevice wifiP2pDevice) {
        if (wifiP2pDevice == null) {
            Log.d(TAG, "onDeviceInfoAvailable(): null device!");
            return;
        }

        Log.d(TAG, "onDeviceInfoAvailable(" + wifiP2pDevice.deviceName + "): " + wifiP2pDevice);
        m_native = ngn_session_init(this, wifiP2pDevice.deviceName);
        Log.d(TAG, "onDeviceInfoAvailable got session: " + m_native);

        if (m_onInit != null) {
            final Runnable onInit = m_onInit;
            m_onInit = null;
            onInit.run();
        }
        peerListChanged();
    }

    static {
        System.loadLibrary("ngn");
    }

    public NgnSessionProxy(Context aContext, NgnListener aListener) {
        ngn_init();
        m_context = aContext;
        m_intentFilter = new IntentFilter();
        m_intentFilter.addAction(WifiP2pManager.WIFI_P2P_STATE_CHANGED_ACTION);
        m_intentFilter.addAction(WifiP2pManager.WIFI_P2P_PEERS_CHANGED_ACTION);
        m_intentFilter.addAction(WifiP2pManager.WIFI_P2P_CONNECTION_CHANGED_ACTION);
        m_intentFilter.addAction(WifiP2pManager.WIFI_P2P_THIS_DEVICE_CHANGED_ACTION);
        m_listener = aListener;
    }

    /**
     * Initializes the P2P session. This needs to be outside the constructor so that the app can
     * make sure to obtain the right permissions.
     *
     * @param onInit Runnable, what to run once we're initialized.
     */
    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public boolean init(Runnable onInit) {
        if (m_manager != null) {
            return false; // Already initialized or initializing
        }
        m_manager = m_context.getSystemService(WifiP2pManager.class);
        m_onInit = onInit;
        assert m_manager != null;
        initChannel();
        onResume();
        m_manager.requestDeviceInfo(m_channel, this);
        return true;
    }

    public void peerListChanged() {
        if (m_peerList == null || m_native == 0) {
            return;
        }
        final Collection<WifiP2pDevice> deviceList = m_peerList.getDeviceList();
        final ArrayList<Peer> peerArrayList = new ArrayList<>();
        final String[] array = new String[deviceList.size() * 2];
        int i = 0;
        for (WifiP2pDevice device : deviceList) {
            array[i++] = device.deviceName;
            array[i++] = device.deviceAddress;
            peerArrayList.add(new Peer(device.deviceName, device.deviceAddress));
        }
        ngn_session_update_peers(m_native, array);
        m_listener.peersChanged(peerArrayList);
    }

    @Override
    protected void finalize() {
        if (m_native == 0) {
            return; // Already finalized, or not initialized.
        }
        m_manager = null;
        m_channel = null;
        m_peerList = null;
        ngn_session_drop(m_native);
        m_native = 0;
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
    public void discoverPeers(WifiP2pManager.ActionListener listener) {
        m_manager.discoverPeers(m_channel, listener);
    }

    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public void discoverPeers(long aNativePromise) {
        discoverPeers(new ActionListenerNativeAdapter(aNativePromise));
    }

    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public void discoverPeers(Function<Boolean, Void> onFinish) {
        discoverPeers(new ActionListenerFunctionAdapter(onFinish));
    }

    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public void discoverPeers() {
        discoverPeers((WifiP2pManager.ActionListener) null);
    }

    // TODO(emilio): Maybe use byte[] rather than string to pass around mac addresses.
    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public void connectToPeer(String aMacAddress, Function<Boolean, Void> onConnect) {
        connectToPeer(aMacAddress, new ActionListenerFunctionAdapter(onConnect));
    }

    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public void connectToPeer(String aMacAddress, long aNativePromise) {
        connectToPeer(aMacAddress, new ActionListenerNativeAdapter(aNativePromise));
    }

    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public void connectToPeer(String aMacAddress) {
        connectToPeer(aMacAddress, (WifiP2pManager.ActionListener) null);
    }

    @RequiresPermission(allOf = {Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.NEARBY_WIFI_DEVICES})
    public void connectToPeer(String aMacAddress, WifiP2pManager.ActionListener aListener) {
        final WifiP2pConfig.Builder builder = new WifiP2pConfig.Builder();
        builder.setDeviceAddress(MacAddress.fromString(aMacAddress));
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            builder.setGroupClientIpProvisioningMode(WifiP2pConfig.GROUP_CLIENT_IP_PROVISIONING_MODE_IPV6_LINK_LOCAL);
        } else {
            Log.w(TAG, "Client IP provisioning might not use IPv6 link-local addressing!");
        }
        m_manager.connect(m_channel, builder.build(), aListener);
    }

    Context m_context;
    WifiP2pManager m_manager;
    WifiP2pManager.Channel m_channel;
    IntentFilter m_intentFilter;
    Runnable m_onInit;
    WifiP2pDeviceList m_peerList;
    WifiP2pInfo m_connectionInfo;
    WifiP2pGroup m_currentGroup;
    long m_native = 0;
    NgnListener m_listener;
}
