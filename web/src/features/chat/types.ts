export interface ChatMessage {
  role: 'user' | 'assistant'
  content: string
}

export interface WsMessage {
  role?: string
  content?: string
  message?: string
}
