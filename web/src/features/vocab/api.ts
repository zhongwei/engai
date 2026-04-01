import { api } from '@/lib/api-client'
import type { WordDetail, WordsResponse } from './types'

export const listWords = () => api<WordsResponse>('/words')

export const getWord = (word: string) =>
  api<WordDetail>(`/words/${encodeURIComponent(word)}`)
