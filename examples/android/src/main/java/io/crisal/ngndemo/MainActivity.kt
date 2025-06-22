package io.crisal.ngndemo


import androidx.compose.foundation.lazy.items
import android.Manifest
import android.annotation.SuppressLint
import android.content.Context
import android.content.pm.PackageManager
import android.net.wifi.p2p.WifiP2pGroup
import android.net.wifi.p2p.WifiP2pManager
import android.net.wifi.p2p.WifiP2pManager.GroupInfoListener
import android.os.Bundle
import android.util.Log
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.RequiresPermission
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.Placeholder
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import io.crisal.ngn.NgnListener
import io.crisal.ngn.NgnSessionProxy
import io.crisal.ngn.Peer
import io.crisal.ngndemo.ui.theme.NgnDemoTheme


val REQUIRED_PERMISSIONS = arrayOf(
    android.Manifest.permission.ACCESS_WIFI_STATE,
    android.Manifest.permission.CHANGE_WIFI_STATE,
    android.Manifest.permission.NEARBY_WIFI_DEVICES,
    android.Manifest.permission.ACCESS_FINE_LOCATION,
    android.Manifest.permission.ACCESS_COARSE_LOCATION,
);

fun hasAllRequiredPermissions(context: Context): Boolean {
    return REQUIRED_PERMISSIONS.all {
        ContextCompat.checkSelfPermission(context, it) == PackageManager.PERMISSION_GRANTED
    }
}

const val TAG = "MainActivity";

@Composable
fun PeerInfoRow(peer: Peer, activity: MainActivity) {
    Row {
        Button(onClick = {
            activity.connectTo(peer)
        }) {
            Text("${peer.name} (${peer.deviceAddress})")
        }
    }
}

@Composable
fun PlaceholderRow(text: String) {
    Text(text, color = Color.Companion.Gray, fontStyle = FontStyle.Italic)
}

class Listener(val activity: MainActivity) : NgnListener() {
    override fun peersChanged(peers: List<Peer>) {
        super.peersChanged(peers);
        activity.peers = peers;
    }
}

class MainActivity : ComponentActivity() {
    private val m_proxy = NgnSessionProxy(this, Listener(this));

    var peers: List<Peer> = arrayListOf()

    @SuppressLint("MissingPermission")
    private val m_permissionRequest =
        registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions())
        { isGranted ->
            if (isGranted.any { entry -> !entry.value }) {
                Toast.makeText(
                    this, "App can't function without the required permissions",
                    Toast.LENGTH_SHORT
                ).show();
            } else {
                m_proxy.init {
                    Log.d(TAG, "Proxy initialized successfully")
                }
            }
        }

    @SuppressLint("MissingPermission")
    fun connectTo(peer: Peer) {
        m_proxy.connectToPeer(peer.deviceAddress);
    }

    override fun onResume() {
        super.onResume()
        m_proxy.onResume();
    }

    override fun onPause() {
        super.onPause()
        m_proxy.onPause();
    }

    @OptIn(ExperimentalFoundationApi::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        setContent {
            NgnDemoTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    LazyColumn(modifier = Modifier.padding(innerPadding)) {
                        stickyHeader {
                            IconButton(onClick = {
                                @SuppressLint("MissingPermission") // We're literally checking, but the linter is not smart enough it seems?
                                if (hasAllRequiredPermissions(this@MainActivity)) {
                                    m_proxy.init {
                                        Log.d(TAG, "Initialized m_proxy from button");
                                        m_proxy.discoverPeers { success ->
                                            Log.d(TAG, "Initiated discovery: $success")
                                            null
                                        };
                                    }
                                } else {
                                    m_permissionRequest.launch(REQUIRED_PERMISSIONS);
                                }
                            }, modifier = Modifier.padding(innerPadding)) {
                                Icon(Icons.Rounded.Refresh, contentDescription = "Refresh")
                            }
                        }
                        if (peers.isEmpty()) {
                            item {
                                PlaceholderRow("Peers will show up here")
                            }
                        } else {
                            items(peers) { peer ->
                                PeerInfoRow(peer, this@MainActivity)
                            }
                        }
                    }
                }
            }
        }
    }
}
