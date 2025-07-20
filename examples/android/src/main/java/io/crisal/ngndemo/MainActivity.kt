package io.crisal.ngndemo

import android.Manifest
import androidx.compose.foundation.lazy.items
import android.annotation.SuppressLint
import android.content.Context
import android.content.pm.PackageManager
import android.os.Bundle
import androidx.navigation.compose.rememberNavController
import android.util.Log
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.ArrowForward
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
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
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.toRoute
import kotlinx.serialization.Serializable
import io.crisal.ngn.NgnListener
import io.crisal.ngn.NgnSessionProxy
import io.crisal.ngn.Peer
import io.crisal.ngndemo.ui.theme.NgnDemoTheme


val REQUIRED_PERMISSIONS = arrayOf(
    Manifest.permission.ACCESS_WIFI_STATE,
    Manifest.permission.CHANGE_WIFI_STATE,
    Manifest.permission.NEARBY_WIFI_DEVICES,
    Manifest.permission.ACCESS_FINE_LOCATION,
    Manifest.permission.ACCESS_COARSE_LOCATION,
)

fun hasAllRequiredPermissions(context: Context): Boolean {
    return REQUIRED_PERMISSIONS.all {
        ContextCompat.checkSelfPermission(context, it) == PackageManager.PERMISSION_GRANTED
    }
}

const val TAG = "MainActivity"

@Composable
fun PeerInfoRow(peer: Peer, activity: MainActivity, onGameStart: (Peer) -> Unit) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable {
                if (peer.logicalId == null) {
                    activity.connectTo(peer)
                } else {
                    onGameStart(peer)
                }
            }) {
        Text(buildAnnotatedString {
            val physicalId = "${peer.name} (${peer.deviceAddress})"
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

@Serializable
object PeerList

@Serializable
data class Game(val peerId: String)

class MainActivity : ComponentActivity() {
    private val proxy = NgnSessionProxy(this, Listener(this))

    val peers = mutableStateOf<List<Peer>>(arrayListOf())
    val identity: MutableState<String?> = mutableStateOf(null)

    val gameBoards: HashMap<String, GameBoard> = hashMapOf()

    @SuppressLint("MissingPermission")
    private val permissionRequest =
        registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { isGranted ->
            if (isGranted.any { entry -> !entry.value }) {
                Toast.makeText(
                    this, "App can't function without the required permissions", Toast.LENGTH_SHORT
                ).show()
            } else {
                this.initProxy(requestPermissions = false)
            }
        }

    // We're literally checking, if needed, but the linter is not smart enough it seems?
    @SuppressLint("MissingPermission")
    fun initProxy(requestPermissions: Boolean) {
        if (requestPermissions && !hasAllRequiredPermissions(this)) {
            permissionRequest.launch(REQUIRED_PERMISSIONS)
            return
        }
        proxy.init(identity.value!!) {
            Log.d(TAG, "Proxy initialized successfully")
            proxy.discoverPeers { success ->
                Log.d(TAG, "Initiated discovery: $success")
                null
            }
        }
    }

    @SuppressLint("MissingPermission")
    fun connectTo(peer: Peer) {
        proxy.connectToPeer(peer.deviceAddress) { success ->
            Log.d(TAG, "connected to $peer: $success")
            null
        }
    }

    @SuppressLint("MissingPermission")
    fun sendMessage(peer: Peer, message: ByteArray) {
        proxy.messagePeer(peer.deviceAddress, message) { success ->
            Log.d(TAG, "sent message to $peer: $success")
            null
        }
    }

    override fun onResume() {
        super.onResume()
        proxy.onResume()
    }

    override fun onPause() {
        super.onPause()
        proxy.onPause()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        setContent {
            NgnDemoTheme {
                val navController = rememberNavController()
                NavHost(navController = navController, startDestination = PeerList) {
                    composable<PeerList> {
                        PeerListPage(this@MainActivity, onGameStart = { peer ->
                            navController.navigate(route = Game(peer.deviceAddress))
                        })
                    }
                    composable<Game> { backStackEntry ->
                        GamePage(
                            this@MainActivity,
                            backStackEntry.toRoute<Game>().peerId,
                            onBack = {
                                navController.navigate(route = PeerList)
                            })
                    }
                }
            }
        }
    }
}

@Composable
fun Page(
    activity: MainActivity,
    topBarReadOnly: Boolean,
    onBack: (() -> Unit)? = null,
    content: @Composable (PaddingValues) -> Unit,
) {
    Scaffold(
        modifier = Modifier.fillMaxSize(), topBar = {
            TopBar(
                activity,
                topBarReadOnly,
                onBack,
            )
        }, content = content
    )
}

@Composable
@OptIn(ExperimentalMaterial3Api::class)
fun TopBar(activity: MainActivity, readOnly: Boolean, onBack: (() -> Unit)? = null) {
    var nickname by rememberSaveable { mutableStateOf("") }
    TopAppBar(modifier = Modifier.shadow(elevation = 10.dp), title = {
        if (activity.identity.value == null) {
            TextField(
                nickname,
                onValueChange = {
                    nickname = it
                },
                modifier = Modifier.fillMaxSize(),
                readOnly = readOnly,
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
                placeholder = {
                    Text("Nickname")
                })
        } else {
            Text(activity.identity.value!!)
        }
    }, navigationIcon = {
        if (onBack != null) {
            IconButton(onClick = { onBack() }) {
                Icon(
                    imageVector = Icons.AutoMirrored.Rounded.ArrowBack,
                    contentDescription = "Localized description"
                )
            }
        }
    }, actions = {
        if (readOnly) {
            return@TopAppBar
        }
        IconButton(onClick = {
            if (nickname == "") {
                return@IconButton
            }
            if (activity.identity.value == null) {
                activity.identity.value = nickname
            }
            activity.initProxy(requestPermissions = true)
        }) {
            if (activity.identity.value == null) {
                Icon(
                    Icons.AutoMirrored.Rounded.ArrowForward, contentDescription = "Start"
                )
            } else {
                Icon(Icons.Rounded.Refresh, contentDescription = "Refresh")
            }
        }
    })
}

@Composable
fun PeerListPage(activity: MainActivity, onGameStart: (Peer) -> Unit) {
    Page(activity, topBarReadOnly = false) { innerPadding ->
        LazyColumn(modifier = Modifier.padding(innerPadding)) {
            if (activity.peers.value.isEmpty()) {
                item {
                    PlaceholderRow("Peers will show up here")
                }
            } else {
                items(activity.peers.value) { peer ->
                    PeerInfoRow(peer, activity, onGameStart)
                }
            }
        }
    }
}

@Composable
fun GamePage(activity: MainActivity, peerId: String, onBack: () -> Unit) {
    Page(activity, topBarReadOnly = true, onBack = onBack) { innerPadding ->
        Box(modifier = Modifier.padding(innerPadding)) {
            Text("Game page here!")
        }
    }
}
