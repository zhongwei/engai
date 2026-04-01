import { useQuery } from '@tanstack/react-query'
import { listReadings, getReading } from './api'

export const useReadings = () =>
  useQuery({ queryKey: ['readings'], queryFn: listReadings })

export const useReading = (id: number | null) =>
  useQuery({ queryKey: ['reading', id], queryFn: () => getReading(id!), enabled: !!id })
