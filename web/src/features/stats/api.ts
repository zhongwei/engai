import { api } from '@/lib/api-client'
import type { Stats } from './types'

export const getStats = () => api<Stats>('/stats')
