import { createFileRoute } from '@tanstack/react-router'
import WordCard from '@/pages/WordCard'

export const Route = createFileRoute('/words/$word')({
  component: WordCard,
})
