package io.crisal.ngndemo


import android.annotation.SuppressLint
import io.crisal.ngn.NgnSessionProxy
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import io.crisal.ngndemo.ui.theme.NgnDemoTheme

val REQUIRED_PERMISSIONS = arrayOf(
    android.Manifest.permission.ACCESS_WIFI_STATE,
    android.Manifest.permission.CHANGE_WIFI_STATE,
    android.Manifest.permission.NEARBY_WIFI_DEVICES,
    android.Manifest.permission.ACCESS_FINE_LOCATION,
    android.Manifest.permission.ACCESS_COARSE_LOCATION,
);

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
        setContent {
            NgnDemoTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Greeting(
                        name = "Android",
                        modifier = Modifier.padding(innerPadding)
                    )
                    Button(onClick = {
                        m_permissionRequest.launch(REQUIRED_PERMISSIONS);
                    }) {
                        Text("Request permissions")
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
