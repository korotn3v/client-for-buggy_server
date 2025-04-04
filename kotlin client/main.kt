import java.net.HttpURLConnection
import java.net.URL
import java.io.ByteArrayOutputStream
import java.io.InputStream
import java.security.MessageDigest
import kotlin.system.exitProcess

fun main(args: Array<String>) {
    if (args.size != 2) {
        System.err.println("Usage: <number_of_bytes> <expected_sha256_hash>")
        exitProcess(1)
    }
    val totalBytes = args[0].toLongOrNull() ?: run {
        System.err.println("Invalid number of bytes: ${args[0]}")
        exitProcess(1)
    }
    val expectedHash = args[1].lowercase()

    val url = "http://127.0.0.1:8080/"
    val chunkSize = 64 * 1024L // 64 KB
    var currentOffset = 0L
    val dataBuffer = ByteArrayOutputStream()

    while (currentOffset < totalBytes) {
        val remaining = totalBytes - currentOffset
        val requestSize = if (remaining < chunkSize) remaining else chunkSize
        val rangeHeader = "bytes=$currentOffset-${currentOffset + requestSize}"
        println("Ask for range: $rangeHeader")

        val connection = URL(url).openConnection() as HttpURLConnection
        connection.requestMethod = "GET"
        connection.setRequestProperty("Range", rangeHeader)
        connection.connectTimeout = 10000
        connection.readTimeout = 10000

        val responseCode = connection.responseCode
        if (responseCode != 200 && responseCode != 206) {
            System.err.println("Error: status code $responseCode")
            exitProcess(1)
        }

        val inputStream: InputStream = connection.inputStream
        val chunk = inputStream.readBytes()
        inputStream.close()
        connection.disconnect()

        val received = chunk.size
        if (received == 0) {
            System.err.println("Get 0 bytes, end the download.")
            break
        }
        dataBuffer.write(chunk)
        currentOffset += received
        println("Get $received bytes, summary $currentOffset/$totalBytes bytes")
    }
    println("Total get: ${dataBuffer.size()} bytes")

    val data = dataBuffer.toByteArray()

    val digest = MessageDigest.getInstance("SHA-256")
    digest.update(data)
    val computedHash = digest.digest().joinToString("") { "%02x".format(it) }

    println("computed_hash SHA-256: $computedHash")
    println("expected_hash SHA-256: $expectedHash")
    if (computedHash == expectedHash) {
        println("data is correct!")
    } else {
        println("error in data!")
    }
}

