package com.example.my_ws_chat_client.login

import android.content.SharedPreferences
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.viewModels
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp
import androidx.lifecycle.lifecycleScope
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.example.my_ws_chat_client.MainActivity
import com.example.my_ws_chat_client.ui.theme.MywschatclientTheme


class LoginActivity : ComponentActivity() {

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val viewModel: LoginViewModel by viewModels()

        sharedPreferences().let { sharedPreferences ->
            val jwt: String? = sharedPreferences.getString("jwt", null)
            if (jwt != null) {
                startMainActivity(jwt)
            }
        }

        fun login(nickname: String, password: String) = lifecycleScope.launchWhenStarted {
            viewModel.login(nickname, password).getOrNull()
                ?.let { jwt ->
                    sharedPreferences().edit()
                        .putString("jwt", jwt)
                        .apply()
                    startMainActivity(jwt)
                }
                ?: Toast.makeText(
                    this@LoginActivity,
                    "Error trying to login!",
                    Toast.LENGTH_SHORT
                ).show()
        }

        fun register(nickname: String, password: String) = lifecycleScope.launchWhenStarted {
            if (viewModel.register(nickname, password)) {
                login(nickname, password)
            } else {
                Toast.makeText(
                    this@LoginActivity,
                    "Error trying to create account!",
                    Toast.LENGTH_SHORT
                ).show()
            }
        }

        setContent {
            MywschatclientTheme {
                Surface(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(10.dp),
                    color = MaterialTheme.colorScheme.background,
                ) {
                    Column(
                        verticalArrangement = Arrangement.Center,
                        horizontalAlignment = Alignment.CenterHorizontally
                    ) {
                        var nicknameValue by remember { mutableStateOf(TextFieldValue("")) }
                        var passwordValue by remember { mutableStateOf(TextFieldValue("")) }
                        TextField(
                            value = nicknameValue,
                            onValueChange = { nicknameValue = it },
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(5.dp),
                            label = { Text(text = "Nickname") },
                        )
                        TextField(
                            value = passwordValue,
                            onValueChange = { passwordValue = it },
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(5.dp),
                            label = { Text(text = "Password") },
                            visualTransformation = PasswordVisualTransformation()
                        )
                        Button(
                            onClick = { register(nicknameValue.text, passwordValue.text) },
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(5.dp).padding(top = 16.dp),
                        ) {
                            Text(text = "Register")
                        }
                        Button(
                            onClick = { login(nicknameValue.text, passwordValue.text) },
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(5.dp),
                        ) {
                            Text(text = "Login")
                        }
                    }
                }
            }
        }
    }

    private fun sharedPreferences(): SharedPreferences {
        val masterKey: MasterKey = MasterKey.Builder(applicationContext)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()

        return EncryptedSharedPreferences.create(
            applicationContext,
            "secret_shared_prefs",
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
        )
    }

    private fun startMainActivity(jwt: String) {
        val intent = MainActivity.intent(this, jwt)
        startActivity(intent)
        finish()
    }
}