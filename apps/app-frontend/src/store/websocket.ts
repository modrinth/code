import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import WebSocket from '@tauri-apps/plugin-websocket'
import { useServerStore } from './servers'
import type { WSEvent, ServerState, Stats } from '@/types/servers'

interface ServerConnection {
  socket: WebSocket | null
  connectionState: 'connecting' | 'connected' | 'disconnected' | 'auth-failed' | 'error'
  consoleOutput: string[]
  serverPowerState: ServerState
  isActioning: boolean
  errorTitle: string
  errorMessage: string
  currentSession: string
  stats: Stats
  messageHandler?: (msg: unknown) => void
}

export const useWebSocketStore = defineStore('webSocket', () => {
  const serverStore = useServerStore()
  const connections = ref<Record<string, ServerConnection>>({})

  function getOrCreateConnection(serverId: string): ServerConnection {
    if (!connections.value[serverId]) {
      connections.value[serverId] = {
        socket: null,
        connectionState: 'disconnected',
        consoleOutput: [],
        serverPowerState: 'stopped',
        isActioning: false,
        errorTitle: '',
        errorMessage: '',
        currentSession: '',
        stats: {
          current: {
            cpu_percent: 0,
            ram_usage_bytes: 0,
            ram_total_bytes: 1,
            storage_usage_bytes: 0,
            storage_total_bytes: 0,
          },
          past: {
            cpu_percent: 0,
            ram_usage_bytes: 0,
            ram_total_bytes: 1,
            storage_usage_bytes: 0,
            storage_total_bytes: 0,
          },
          graph: {
            cpu: [],
            ram: [],
          },
        },
      }
    }
    return connections.value[serverId]
  }

  const isConnected = computed(
    () => (serverId: string) => getOrCreateConnection(serverId).connectionState === 'connected',
  )

  async function connect(serverId: string, session: string) {
    const connection = getOrCreateConnection(serverId)

    if (connection.connectionState === 'connected' || connection.connectionState === 'connecting') {
      console.log(`Already connected or connecting to server ${serverId}`)
      return
    }

    try {
      await disconnect(serverId)

      connection.connectionState = 'connecting'
      connection.currentSession = session
      const wsAuth = await serverStore.requestWebsocket(session, serverId)

      const newSocket = await WebSocket.connect(`wss://${wsAuth.url}`)

      if (connection.socket) {
        return
      }

      const messageHandler = (msg: unknown) => handleWebSocketMessage(serverId, msg as WSEvent)
      newSocket.addListener(messageHandler)

      connection.socket = newSocket
      connection.messageHandler = messageHandler

      await connection.socket.send(JSON.stringify({ event: 'auth', jwt: wsAuth.token }))

      connection.connectionState = 'connected'
      console.log(`Connected to server ${serverId}`)
    } catch (error) {
      console.error(`Failed to connect to WebSocket for server ${serverId}:`, error)
      handleConnectionError(serverId, error)
    }
  }

  function handleWebSocketMessage(serverId: string, message: WSEvent) {
    const connection = getOrCreateConnection(serverId)
    try {
      if (typeof message === 'object' && message !== null && 'data' in message) {
        const data = JSON.parse(message.data as string)

        switch (data.event) {
          case 'log':
            connection.consoleOutput.push(data.message)
            break
          case 'stats':
            updateStats(serverId, data as unknown as Stats['current'])
            break
          case 'auth-expiring':
            reauth(serverId)
            break
          case 'power-state':
            updatePowerState(serverId, data.state)
            break
          case 'auth-incorrect':
            connection.connectionState = 'auth-failed'
            break
          default:
            console.warn('Unhandled WebSocket event:', data.event)
        }
      } else {
        console.warn('Unexpected WebSocket message format:', message)
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', message, error)
    }
  }

  async function disconnect(serverId: string) {
    const connection = getOrCreateConnection(serverId)
    if (connection.socket) {
      try {
        await connection.socket.disconnect()
      } catch (error) {
        console.error('Error disconnecting WebSocket:', error)
      }
      connection.socket = null
      connection.connectionState = 'disconnected'
      connection.currentSession = ''
    }
  }

  function updatePowerState(serverId: string, state: ServerState) {
    const connection = getOrCreateConnection(serverId)
    connection.serverPowerState = state
  }

  function updateStats(serverId: string, data: Stats['current']) {
    const connection = getOrCreateConnection(serverId)
    connection.stats = {
      current: data,
      past: connection.stats.current,
      graph: {
        cpu: updateDataArray(connection.stats.graph.cpu, Math.round(data.cpu_percent * 100) / 100),
        ram: updateDataArray(
          connection.stats.graph.ram,
          Math.floor((data.ram_usage_bytes / data.ram_total_bytes) * 100),
        ),
      },
    }
  }

  function updateDataArray(arr: number[], newValue: number) {
    const newArr = [...arr, newValue]
    if (newArr.length > 10) newArr.shift()
    return newArr
  }

  async function reauth(serverId: string) {
    const connection = getOrCreateConnection(serverId)
    try {
      const wsAuth = await serverStore.requestWebsocket(connection.currentSession, serverId)
      await connection.socket?.send(JSON.stringify({ event: 'auth', jwt: wsAuth.token }))
    } catch (error) {
      console.error('Failed to reauthenticate:', error)
      handleConnectionError(serverId, error)
    }
  }

  async function sendPowerAction(auth: string, serverId: string, action: 'restart' | 'start' | 'stop' | 'kill') {
    const connection = getOrCreateConnection(serverId)
    const actionName = action.charAt(0).toUpperCase() + action.slice(1)
    console.log(`${actionName}ing server ${serverId}`)

    try {
      connection.isActioning = true
      await serverStore.sendPowerAction(auth, serverId, actionName)
    } catch (error) {
      console.error(`Error ${actionName}ing server:`, error)
      connection.errorTitle = `Failed to ${actionName.toLowerCase()} server`
      connection.errorMessage = `An error occurred while trying to ${actionName.toLowerCase()} the server. Please try again later.`
      connection.connectionState = 'error'
    } finally {
      connection.isActioning = false
    }
  }

  function handleConnectionError(serverId: string, error: unknown) {
    const connection = getOrCreateConnection(serverId)
    console.error('Connection error:', error)
    connection.errorTitle = 'Connection failed'
    connection.errorMessage =
      'Unable to connect to the server. Please check your internet connection and try again later.'
    connection.connectionState = 'error'
  }

  function clearConsole(serverId: string) {
    const connection = getOrCreateConnection(serverId)
    connection.consoleOutput = []
  }

  return {
    connections,
    isConnected,
    connect,
    disconnect,
    sendPowerAction,
    clearConsole,
    reauth,
  }
})
