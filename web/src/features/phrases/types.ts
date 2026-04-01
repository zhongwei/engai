export interface Phrase {
  id: number
  phrase: string
  meaning: string
  familiarity: number
}

export interface PhrasesResponse {
  phrases: Phrase[]
}
