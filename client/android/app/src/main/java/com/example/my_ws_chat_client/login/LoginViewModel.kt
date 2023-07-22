package com.example.my_ws_chat_client.login

import androidx.lifecycle.ViewModel
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
        post("$BASE_URL/signup", body.toString()).isSuccess
    }

    suspend fun login(nickname: String, password: String): Result<String> = withContext(Dispatchers.IO) {
        val body = prepareBody(nickname, password)
        post("$BASE_URL/login", body.toString())
            .map { JSONObject(it).get("jwt").toString() }
    }


    private fun prepareBody(nickname: String, password: String): JSONObject {
        val jsonBody = JSONObject().apply {
            put("nickname", nickname)
            put("password", password)
        }
        return jsonBody
    }

    private fun post(url: String, json: String): Result<String> = runCatching {
        val body: RequestBody = json.toRequestBody(JSON)
        val request: Request = Request.Builder()
            .url(url)
            .post(body)
            .build()

        client.newCall(request)
            .execute()
            .use { response -> response.body?.string().orEmpty() }
    }

    companion object {
        const val BASE_URL = "http://10.0.2.2:3000"
        val JSON: MediaType = "application/json".toMediaType()
    }
}