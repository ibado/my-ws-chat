package com.example.my_ws_chat_client.login

import android.os.Bundle
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
import com.example.my_ws_chat_client.MainActivity
import com.example.my_ws_chat_client.login.LoginViewModel.LoginResult
import com.example.my_ws_chat_client.sharedPreferences
import com.example.my_ws_chat_client.showToast
import com.example.my_ws_chat_client.ui.theme.MywschatclientTheme
import okio.ByteString.Companion.decodeBase64
import org.json.JSONObject
import java.time.Instant
import java.time.LocalDateTime
import java.time.ZoneId


class LoginActivity : ComponentActivity() {

    @OptIn(ExperimentalMaterial3Api::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val viewModel: LoginViewModel by viewModels()

        sharedPreferences().let { sharedPreferences ->
            val jwt: String? = sharedPreferences.getString("jwt", null)
            if (jwt != null && !jwt.isExpired()) {
                startMainActivity(jwt)
            } else {
                sharedPreferences.edit()
                    .remove("jwt")
                    .apply()
            }
        }

        fun login(nickname: String, password: String) = lifecycleScope.launchWhenStarted {
            when (val result = viewModel.login(nickname, password)) {
                is LoginResult.Failure -> showToast(result.message)

                is LoginResult.Success -> {
                    sharedPreferences().edit()
                        .putString("jwt", result.jwt)
                        .apply()
                    startMainActivity(result.jwt)
                }
            }
        }

        fun register(nickname: String, password: String) = lifecycleScope.launchWhenStarted {
            if (viewModel.register(nickname, password)) {
                login(nickname, password)
            } else {
                showToast("Error trying to create account!")
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
                                .padding(5.dp)
                                .padding(top = 16.dp),
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

    private fun String.isExpired(): Boolean =
        split(".")[1]
            .decodeBase64()?.utf8()
            ?.let(::JSONObject)
            ?.getString("exp")
            ?.let { Instant.ofEpochSecond(it.toLong()) }
            ?.let { exp ->
                val expDay = LocalDateTime.ofInstant(exp, ZoneId.of("UTC"))
                val today = LocalDateTime.now().atZone(ZoneId.of("UTC"))
                expDay <= today.toLocalDateTime()
            } ?: false


    private fun startMainActivity(jwt: String) {
        val intent = MainActivity.intent(this, jwt)
        startActivity(intent)
        finish()
    }
}