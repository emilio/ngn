package io.crisal.ngndemo


import androidx.compose.foundation.lazy.items
import android.annotation.SuppressLint
import android.content.Context
import android.content.pm.PackageManager
import android.os.Bundle
import android.text.SpannableString
import android.util.Log
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowForward
import androidx.compose.material.icons.filled.ArrowForward
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SearchBarColors
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.sp
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import io.crisal.ngn.ConnectionState
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
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable {
                if (peer.logicalId == null) {
                    activity.connectTo(peer)
                } else {
                    activity.sendMessage(peer, "foobar".toByteArray())
                }
            }) {
        Text(buildAnnotatedString {
            val physicalId = "${peer.name} (${peer.deviceAddress})";
            if (peer.logicalId != null) {
                append(peer.logicalId!!)
                append("\n")
                withStyle(SpanStyle(fontSize = 12.sp)) {
                    append(physicalId)
                }
            } else {
                append(physicalId)
            }
        }, modifier = Modifier.padding(8.dp))
    }
}

@Composable
fun PlaceholderRow(text: String) {
    Text(text, color = Color.Companion.Gray, fontStyle = FontStyle.Italic)
}

class Listener(val activity: MainActivity) : NgnListener() {
    override fun peersChanged(peers: List<Peer>) {
        super.peersChanged(peers)
        activity.peers.value = peers
    }

    override fun messageReceived(from: Peer, content: ByteArray) {
        super.messageReceived(from, content)
        activity.runOnUiThread {
            Toast.makeText(activity, "Message from ${from}: ${String(content)}", Toast.LENGTH_LONG)
                .show()
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
class MainActivity : ComponentActivity() {
    private val m_proxy = NgnSessionProxy(this, Listener(this));

    val peers = mutableStateOf<List<Peer>>(arrayListOf())
    val identity: MutableState<String?> = mutableStateOf(null);

    @SuppressLint("MissingPermission")
    private val m_permissionRequest =
        registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { isGranted ->
            if (isGranted.any { entry -> !entry.value }) {
                Toast.makeText(
                    this, "App can't function without the required permissions", Toast.LENGTH_SHORT
                ).show();
            } else {
                m_proxy.init(identity.value!!) {
                    Log.d(TAG, "Proxy initialized successfully")
                }
            }
        }

    @SuppressLint("MissingPermission")
    fun connectTo(peer: Peer) {
        m_proxy.connectToPeer(peer.deviceAddress) { success ->
            Log.d(TAG, "connected to $peer: $success");
            null
        };
    }

    @SuppressLint("MissingPermission")
    fun sendMessage(peer: Peer, message: ByteArray) {
        m_proxy.messagePeer(peer.deviceAddress, message) { success ->
            Log.d(TAG, "sent message to $peer: $success");
            null
        }
    }

    override fun onResume() {
        super.onResume()
        m_proxy.onResume();
    }

    override fun onPause() {
        super.onPause()
        m_proxy.onPause();
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        setContent {
            NgnDemoTheme {
                Scaffold(
                    modifier = Modifier.fillMaxSize(), topBar = {
                        var nickname by rememberSaveable { mutableStateOf("") };
                        TopAppBar(
                            modifier = Modifier.shadow(elevation = 10.dp),
                            title = {
                                if (identity.value == null) {
                                    TextField(
                                        nickname,
                                        modifier = Modifier.fillMaxSize(),
                                        singleLine = true,
                                        colors = TextFieldDefaults.colors(
                                            focusedIndicatorColor = Color.Transparent,
                                            unfocusedIndicatorColor = Color.Transparent,
                                            disabledIndicatorColor = Color.Transparent,
                                            errorIndicatorColor = Color.Transparent,
                                            focusedContainerColor = Color.Transparent,
                                            unfocusedContainerColor = Color.Transparent,
                                            disabledContainerColor = Color.Transparent,
                                            errorContainerColor = Color.Transparent
                                        ),
                                        onValueChange = {
                                            nickname = it
                                        },
                                        placeholder = {
                                            Text("Nickname")
                                        },
                                    )
                                } else {
                                    Text(identity.value!!)
                                }
                            }, actions = {
                                IconButton(onClick = {
                                    if (identity.value == null) {
                                        if (nickname == "") {
                                            return@IconButton
                                        }
                                        identity.value = nickname;
                                    }
                                    @SuppressLint("MissingPermission") // We're literally checking, but the linter is not smart enough it seems?
                                    if (hasAllRequiredPermissions(this@MainActivity)) {
                                        m_proxy.init(identity.value!!) {
                                            Log.d(TAG, "Initialized m_proxy from button");
                                            m_proxy.discoverPeers { success ->
                                                Log.d(TAG, "Initiated discovery: $success")
                                                null
                                            };
                                        }
                                    } else {
                                        m_permissionRequest.launch(REQUIRED_PERMISSIONS);
                                    }
                                }) {
                                    if (identity.value == null) {
                                        Icon(
                                            Icons.AutoMirrored.Rounded.ArrowForward,
                                            contentDescription = "Start"
                                        )
                                    } else {
                                        Icon(Icons.Rounded.Refresh, contentDescription = "Refresh")
                                    }
                                }
                            })
                    }) { innerPadding ->
                    LazyColumn(modifier = Modifier.padding(innerPadding)) {
                        if (peers.value.isEmpty()) {
                            item {
                                PlaceholderRow("Peers will show up here")
                            }
                        } else {
                            items(peers.value) { peer ->
                                PeerInfoRow(peer, this@MainActivity)
                            }
                        }
                    }
                }
            }
        }
    }
}
