export interface Reading {
  id: number
  title: string
  content?: string
  source?: string
  created_at: string
}

export interface ReadingDetail {
  id: number
  title: string
  content: string
  source?: string
  created_at: string
}

export interface ReadingsResponse {
  readings: Reading[]
}
