package io.crisal.ngndemo

import android.Manifest
import androidx.compose.foundation.lazy.items
import android.annotation.SuppressLint
import android.content.Context
import android.content.pm.PackageManager
import android.graphics.Color.argb
import android.os.Bundle
import androidx.navigation.compose.rememberNavController
import android.util.Log
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.ArrowBack
import androidx.compose.material.icons.automirrored.rounded.ArrowForward
import androidx.compose.material.icons.rounded.Refresh
import androidx.compose.material3.BottomAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalViewConfiguration
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.sp
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.toRoute
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import io.crisal.ngn.NgnListener
import io.crisal.ngn.NgnSessionProxy
import io.crisal.ngn.Peer
import io.crisal.ngndemo.ui.theme.NgnDemoTheme
import kotlin.math.abs
import kotlin.math.max


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

// move=null starts a match, otherwise performs a move on the pre-existing match
@Serializable
class Message(val tile: Tile?, val move: MoveDirection?)

class Listener(val activity: MainActivity) : NgnListener() {
    override fun peersChanged(peers: List<Peer>) {
        super.peersChanged(peers)
        activity.runOnUiThread {
            activity.peersChanged(peers)
        }
    }

    override fun messageReceived(from: Peer, content: ByteArray) {
        super.messageReceived(from, content)
        activity.runOnUiThread {
            try {
                val message = Json.decodeFromString<Message>(String(content))
                activity.handleMessage(from, message)
            } catch (e: Exception) {
                Log.e(TAG, "Message from $from: ${String(content)} couldn't be handled: $e")
                Toast.makeText(
                    activity,
                    "Message from $from ${String(content)} couldn't be decoded to a message",
                    Toast.LENGTH_LONG
                ).show()
            }
        }
    }
}

@Serializable
object PeerList

@Serializable
object SinglePlayerGame

@Serializable
data class MultiplayerGame(val peerId: String)

class MainActivity : ComponentActivity() {
    private val proxy = NgnSessionProxy(this, Listener(this))

    val peers = mutableStateOf<List<Peer>>(arrayListOf())
    val identity: MutableState<String?> = mutableStateOf(null)
    val observers: ArrayList<(Peer, Message) -> Unit> = arrayListOf()

    fun peersChanged(newPeers: List<Peer>) {
        peers.value = newPeers
        // Remove the boards that we can't reach anymore.
        gameBoards.keys.retainAll(newPeers.map { p -> p.deviceAddress })
    }

    fun boardFor(peer: String): GameBoard {
        return gameBoards.getOrPut(peer) { GameBoard() }
    }

    fun handleMessage(peer: Peer, message: Message) {
        Log.d(TAG, "handleMessage($peer, $message")
        val board = boardFor(peer.deviceAddress)
        if (message.move != null) {
            board.tryMove(message.move, tileToCreate = message.tile)
        } else if (message.tile != null) {
            board.startWithTile(message.tile)
        }
        for (observer in ArrayList(observers)) {
            observer(peer, message)
        }
    }

    val gameBoards: HashMap<String, GameBoard> = hashMapOf()

    val singleGameBoard = GameBoard()

    @SuppressLint("MissingPermission")
    private val permissionRequest =
        registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) { isGranted ->
            if (isGranted.any { entry -> !entry.value }) {
                Toast.makeText(
                    this, "App can't function without the required permissions", Toast.LENGTH_SHORT
                ).show()
            } else {
                initProxy(requestPermissions = false)
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
    fun sendMessage(peerDevAddr: String, message: ByteArray) {
        proxy.messagePeer(peerDevAddr, message) { success ->
            Log.d(TAG, "sent message to $peerDevAddr: $success")
            null
        }
    }

    fun sendMessage(peerDevAddr: String, message: Message) {
        val bytes = Json.encodeToString(message).toByteArray()
        sendMessage(peerDevAddr, bytes)
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
                DisposableEffect(Unit) {
                    val messageObserver: (Peer, Message) -> Unit = { peer, message ->
                        if (message.move == null) {
                            val board = boardFor(peer.deviceAddress)
                            val isNewGame = board.tiles.size == 1
                            if (isNewGame) {
                                navController.navigate(route = MultiplayerGame(peer.deviceAddress))
                            }
                        }
                    }
                    observers.add(messageObserver)
                    onDispose {
                        observers.remove(messageObserver)
                    }
                }

                NavHost(navController = navController, startDestination = PeerList) {
                    composable<PeerList> {
                        PeerListPage(this@MainActivity, onSingleGame = {
                            navController.navigate(route = SinglePlayerGame)
                        }, onGameStart = { peer ->
                            navController.navigate(route = MultiplayerGame(peer.deviceAddress))
                        })
                    }
                    composable<SinglePlayerGame> {
                        GamePage(this@MainActivity, peerId = null, onBack = {
                            navController.navigate(route = PeerList)
                        })
                    }
                    composable<MultiplayerGame> { backStackEntry ->
                        GamePage(
                            this@MainActivity,
                            backStackEntry.toRoute<MultiplayerGame>().peerId,
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
    onSingleGame: (() -> Unit)? = null,
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
    }, bottomBar = {
        if (onSingleGame != null) {
            BottomAppBar(contentPadding = PaddingValues(0.dp)) {
                TextButton(modifier = Modifier.fillMaxWidth(), onClick = onSingleGame) {
                    Text("Play single game")
                }
            }
        }
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
fun PeerListPage(activity: MainActivity, onSingleGame: () -> Unit, onGameStart: (Peer) -> Unit) {
    Page(activity, onSingleGame = onSingleGame, topBarReadOnly = false) { innerPadding ->
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

val COLORS: Array<Int> = arrayOf(
    argb(100, 0, 0, 200),
    argb(125, 0, 0, 200),
    argb(150, 0, 0, 200),
    argb(175, 0, 0, 200),
    argb(200, 0, 0, 200),
    argb(225, 0, 0, 200),
    argb(250, 0, 0, 200),
)

fun backgroundForValue(value: Int): Color {
    if (value == 0) {
        return Color.Gray
    }
    return Color(COLORS[value.countTrailingZeroBits() % COLORS.size])
}

@Composable
fun GamePage(activity: MainActivity, peerId: String?, onBack: () -> Unit) {
    val board = if (peerId == null) {
        activity.singleGameBoard
    } else {
        activity.boardFor(peerId)
    }
    // A bit hacky, could be cleaner with something as https://developer.android.com/develop/ui/compose/side-effects
    var state by rememberSaveable { mutableStateOf(board.state) }
    var tiles by rememberSaveable { mutableStateOf(board.cellValues()) }
    val refreshBoardState = {
        state = board.state
        tiles = board.cellValues()
    }
    val reset = {
        board.reset()
        board.start()
        if (peerId != null) {
            val tile = board.tiles[0]
            activity.sendMessage(
                peerId, Message(tile = tile, move = null)
            )
        }
        refreshBoardState()
    }

    if (board.state == GameState.NotYetStarted) {
        reset()
    }

    DisposableEffect(Unit) {
        val messageObserver: (Peer, Message) -> Unit = { peer, message ->
            if (peerId == peer.deviceAddress) {
                refreshBoardState()
            }
        }
        activity.observers.add(messageObserver)
        onDispose {
            activity.observers.remove(messageObserver)
        }
    }

    Page(activity, topBarReadOnly = true, onBack = onBack) { innerPadding ->
        Column(Modifier.padding(innerPadding)) {
            Text(
                "Score: ${tiles.sum()}, state: $state",
                textAlign = TextAlign.Center,
                modifier = Modifier
                    .padding(8.dp)
                    .fillMaxWidth()
            )

            var totalDragDistance by remember { mutableStateOf(Offset.Zero) }
            val config = LocalViewConfiguration.current
            val modifier = Modifier
                .pointerInput(Unit) {
                    detectDragGestures(onDragEnd = {
                        if (state != GameState.MyTurn && (state != GameState.OtherTurn || peerId != null)) {
                            return@detectDragGestures
                        }
                        val (dx, dy) = totalDragDistance
                        val horizontal = abs(dx) > abs(dy)
                        val maxDistance = max(abs(dx), abs(dy))
                        if (maxDistance < config.touchSlop) {
                            Log.d(
                                TAG, "ignore drag of $maxDistance due to ${config.touchSlop} slop"
                            )
                            return@detectDragGestures
                        }
                        val direction = if (horizontal) {
                            if (dx > 0) {
                                MoveDirection.Right
                            } else {
                                MoveDirection.Left
                            }
                        } else {
                            if (dy > 0) {
                                MoveDirection.Down
                            } else {
                                MoveDirection.Up
                            }
                        }
                        val tile = board.tryMove(direction)
                        Log.d(TAG, "tryMove($direction): $tile, $state -> ${board.state}")
                        refreshBoardState()
                        if (peerId != null) {
                            // NOTE: we could save some traffic for invalid moves if we knew that tile != null and we still weren't done, but doesn't seem worth it.
                            activity.sendMessage(
                                peerId, Message(move = direction, tile = tile)
                            )
                        }
                    }, onDragStart = {
                        totalDragDistance = Offset.Zero
                    }) { change, dragAmount ->
                        change.consume()
                        totalDragDistance += dragAmount
                    }
                }
                .weight(1f)
            Column(
                modifier = modifier,
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(
                    8.dp, alignment = Alignment.CenterVertically
                )
            ) {
                for (x in 0..<board.size) {
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(
                            8.dp, alignment = Alignment.CenterHorizontally
                        )
                    ) {
                        for (y in 0..<board.size) {
                            val item = tiles[x + y * board.size]
                            Box(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(8.dp))
                                    .weight(1f)
                                    .aspectRatio(1f)
                                    .background(backgroundForValue(item)),
                                contentAlignment = Alignment.Center,
                            ) {
                                Text(
                                    item.toString(), color = if (item == 0) {
                                        Color.Transparent
                                    } else {
                                        Color.White
                                    }, style = MaterialTheme.typography.headlineLarge
                                )
                            }
                        }
                    }
                }
            }

            Row {
                IconButton(onClick = {
                    reset()
                }) {
                    Icon(
                        imageVector = Icons.Rounded.Refresh, contentDescription = "Restart"
                    )
                }
            }
        }
    }
}
