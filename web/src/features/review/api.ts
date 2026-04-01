import { api } from '@/lib/api-client'
import type { ReviewTodayResponse, ReviewItem } from './types'

export const getReviewToday = () => api<ReviewTodayResponse>('/review/today')

export const submitReview = (targetType: ReviewItem['target_type'], id: number, quality: number) =>
  api(`/review/${targetType}/${id}`, {
    method: 'POST',
    body: JSON.stringify({ quality }),
  })
