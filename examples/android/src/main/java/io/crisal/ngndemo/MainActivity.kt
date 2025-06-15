package io.crisal.ngndemo


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
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import io.crisal.ngn.NgnSessionProxy
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

class MainActivity : ComponentActivity() {
    private val m_proxy = NgnSessionProxy(this);

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
                m_proxy.init();
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

        @SuppressLint("MissingPermission") // We're literally checking, but the linter is not smart enough it seems?
        if (hasAllRequiredPermissions(this)) {
            m_proxy.init();
        }

        setContent {
            NgnDemoTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    if (!hasAllRequiredPermissions(this)) {
                        Button(onClick = {
                            m_permissionRequest.launch(REQUIRED_PERMISSIONS);
                        }, modifier = Modifier.padding(innerPadding)) {
                            Text("Request permissions")
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun Greeting(name: String, modifier: Modifier = Modifier) {
    Text(
        text = "Hello $name!",
        modifier = modifier
    )
}

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    NgnDemoTheme {
        Greeting("Android")
    }
}
