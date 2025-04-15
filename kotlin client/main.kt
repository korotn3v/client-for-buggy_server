import java.io.ByteArrayOutputStream
import java.io.InputStream
import java.net.Socket
import java.security.MessageDigest
import kotlin.system.exitProcess

fun readLineFromStream(input: InputStream): String? {
    val baos = ByteArrayOutputStream()
    var prev = -1
    while (true) {
        val current = input.read()
        if (current == -1) break
            baos.write(current)
            if (prev == 13 && current == 10) break
                prev = current
    }
    if (baos.size() == 0) return null
        return baos.toString("UTF-8")
}

fun toHexString(bytes: ByteArray): String =
bytes.joinToString("") { "%02x".format(it) }

fun main(args: Array<String>) {
    if (args.size != 2) {
        System.err.println("Usage: <number_of_bytes> <expected_sha256_hash>")
        exitProcess(1)
    }
    val totalBytes = args[0].toIntOrNull() ?: run {
        System.err.println("Invalid number of bytes: ${args[0]}")
        exitProcess(1)
        0
    }
    val expectedHash = args[1].lowercase()

    val host = "127.0.0.1"
    val port = 8080
    var currentOffset = 0
    val dataBuffer = ByteArrayOutputStream()

    while (currentOffset < totalBytes) {
        val remaining = totalBytes - currentOffset
        val requestSize = remaining
        val rangeHeader = "bytes=$currentOffset-${currentOffset + requestSize}"
        println("Ask for range: $rangeHeader")

        val socket = Socket(host, port)
        val output = socket.getOutputStream()
        val request = "GET / HTTP/1.1\r\n" +
        "Host: $host:$port\r\n" +
        "Range: $rangeHeader\r\n" +
        "Connection: close\r\n\r\n"
        output.write(request.toByteArray(Charsets.UTF_8))
        output.flush()

        val input = socket.getInputStream()

        val statusLine = readLineFromStream(input) ?: ""
        val statusParts = statusLine.trim().split(" ")
        if (statusParts.size < 2) {
            System.err.println("Invalid HTTP response: $statusLine")
            socket.close()
            exitProcess(1)
        }
        val statusCode = statusParts[1].toIntOrNull() ?: 0
        if (statusCode != 200 && statusCode != 206) {
            System.err.println("Error: status code $statusCode")
            socket.close()
            exitProcess(1)
        }

        var contentLength: Int? = null
        while (true) {
            val line = readLineFromStream(input) ?: break
            if (line.trim().isEmpty()) break
                if (line.lowercase().startsWith("content-length:")) {
                    val parts = line.split(":")
                    if (parts.size >= 2) {
                        contentLength = parts[1].trim().toIntOrNull()
                    }
                }
        }
        if (contentLength == null) {
            System.err.println("No Content-Length header found.")
            socket.close()
            exitProcess(1)
        }

        val chunk = ByteArray(contentLength)
        var totalRead = 0
        while (totalRead < contentLength) {
            val bytesRead = input.read(chunk, totalRead, contentLength - totalRead)
            if (bytesRead == -1) break
                totalRead += bytesRead
        }
        socket.close()

        if (totalRead == 0) {
            System.err.println("Get 0 bytes, end the download.")
            break
        }

        dataBuffer.write(chunk, 0, totalRead)
        currentOffset += totalRead
        println("Get $totalRead bytes, summary $currentOffset/$totalBytes bytes")
    }

    println("Total get: ${dataBuffer.size()} bytes")

    val digest = MessageDigest.getInstance("SHA-256")
    digest.update(dataBuffer.toByteArray())
    val computedHash = toHexString(digest.digest())
    println("computed_hash SHA-256: $computedHash")
    println("expected_hash SHA-256: $expectedHash")
    if (computedHash == expectedHash) {
        println("data is correct!")
    } else {
        println("error in data!")
    }
}
