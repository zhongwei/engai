import { useQuery } from '@tanstack/react-query'
import { getReviewToday } from './api'

export const useReviewToday = () =>
  useQuery({ queryKey: ['review-today'], queryFn: getReviewToday })
