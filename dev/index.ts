import arachne from '../server'
import { createServer } from "http";

const httpServer = createServer()
const ws = new arachne(httpServer, {
    benchmarks: true
})

ws.on('connection', (connection) => {
    console.log(`New WebSocket connection [${connection.id}]`)
    connection.send('Hello from the server!')

    connection.on('message', (message) => {
        console.log('Received message:', message)
        connection.broadcast(message, true)
    })
})


httpServer.listen(8008, () => {
    console.log('Server is running on http://localhost:8008')
})