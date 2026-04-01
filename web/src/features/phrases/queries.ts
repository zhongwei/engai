import { useQuery } from '@tanstack/react-query'
import { listPhrases } from './api'

export const usePhrases = () =>
  useQuery({ queryKey: ['phrases'], queryFn: listPhrases })
