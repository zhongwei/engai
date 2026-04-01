import { useQuery } from '@tanstack/react-query'
import { getStats } from './api'

export const useStats = () =>
  useQuery({ queryKey: ['stats'], queryFn: getStats })
