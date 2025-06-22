package io.crisal.ngn

import android.util.Log

const val TAG: String = "NgnListener";
data class Peer(val name: String, val deviceAddress: String /* , val id: ULong */);

open class NgnListener {
    open fun peersChanged(peers: List<Peer>) {
        Log.d(TAG, peers.toString());
    }
}
