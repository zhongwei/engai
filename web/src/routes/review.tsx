import { createFileRoute } from '@tanstack/react-router'
import Review from '@/pages/Review'

export const Route = createFileRoute('/review')({
  component: Review,
})
