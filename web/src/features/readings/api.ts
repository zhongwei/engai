import { api } from '@/lib/api-client'
import type { ReadingDetail, ReadingsResponse } from './types'

export const listReadings = () => api<ReadingsResponse>('/readings')

export const getReading = (id: number) => api<ReadingDetail>(`/readings/${id}`)
