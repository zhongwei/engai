export interface ReviewItem {
  target_type: 'word' | 'phrase'
  id: number
  display: string
  familiarity: number
  interval: number
  ease_factor: number
}

export interface ReviewTodayResponse {
  items: ReviewItem[]
}
