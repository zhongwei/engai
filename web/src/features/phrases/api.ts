import { api } from '@/lib/api-client'
import type { PhrasesResponse } from './types'

export const listPhrases = () => api<PhrasesResponse>('/phrases')
