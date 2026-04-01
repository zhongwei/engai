export interface Word {
  word: string
  meaning: string
  phonetic?: string
  familiarity: number
}

export interface WordDetail {
  word: string
  meaning: string
  phonetic?: string
  examples?: string[]
  notes?: string
  familiarity: number
}

export interface CreateWord {
  word: string
  phonetic?: string
  meaning: string
}

export interface WordsResponse {
  words: Word[]
}
