package io.crisal.ngn

import android.util.Log

const val TAG: String = "NgnListener";
data class Peer(val name: String, val deviceAddress: String, val logicalId: String?);

enum class ConnectionState {
    Disconnected,
    Connecting,
    Connected,
}

open class NgnListener {
    open fun peersChanged(peers: List<Peer>) {
        Log.d(TAG, "peersChanged($peers)")
    }

    open fun connectionStateChanged(state: ConnectionState) {
        Log.d(TAG, "connectionStateChanged($state)")
    }

    open fun messageReceived(from: Peer, content: ByteArray) {
        Log.d(TAG, "messageReceived($from, $content)")
    }
}
