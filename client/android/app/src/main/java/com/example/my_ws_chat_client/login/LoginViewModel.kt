package com.example.my_ws_chat_client.login

import android.util.Log
import androidx.lifecycle.ViewModel
import com.example.my_ws_chat_client.BuildConfig
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject

class LoginViewModel : ViewModel() {

    private val client = OkHttpClient()

    suspend fun register(nickname: String, password: String): Boolean = withContext(Dispatchers.IO) {
        val body = prepareBody(nickname, password)
        post("${BuildConfig.BACKEND_BASE_URL}/signup", body.toString()).getOrNull()?.code == 201
    }

    suspend fun login(nickname: String, password: String): LoginResult = withContext(Dispatchers.IO) {
        val body = prepareBody(nickname, password)
        post("${BuildConfig.BACKEND_BASE_URL}/login", body.toString())
            .fold(
                onSuccess = { (status, body) ->
                    when (status) {
                        200 -> LoginResult.Success(body!!.get("jwt").toString())
                        401, 404 -> LoginResult.Failure("Invalid nick or password!")
                        else -> {
                            Log.e(TAG, "Unexpected http status: $status")
                            LoginResult.Failure("Something went wrong, please try later")
                        }
                    }
                },
                onFailure = {
                    Log.e(TAG, "Error tyring to login", it)
                    LoginResult.Failure("Something went wrong :(")
                }
            )
    }

    sealed interface LoginResult {
        @JvmInline value class Success(val jwt: String) : LoginResult
        @JvmInline value class Failure(val message: String) : LoginResult
    }


    private fun prepareBody(nickname: String, password: String): JSONObject {
        val jsonBody = JSONObject().apply {
            put("nickname", nickname)
            put("password", password)
        }
        return jsonBody
    }

    private fun post(url: String, json: String): Result<HttpResponse> = runCatching {
        val body: RequestBody = json.toRequestBody(JSON)
        val request: Request = Request.Builder()
            .url(url)
            .post(body)
            .build()
        client.newCall(request)
            .execute()
            .use { response ->
                val responseBody = response.body?.string()
                    ?.takeUnless { it.isEmpty() }
                    ?.let(::JSONObject)
                HttpResponse(response.code, responseBody)
            }
    }

    private data class HttpResponse(val code: Int, val body: JSONObject?)

    companion object {
        private val TAG = LoginViewModel::class.java.simpleName
        val JSON: MediaType = "application/json".toMediaType()
    }
}