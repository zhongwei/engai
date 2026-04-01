import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { apiFetch } from '@/lib/api'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { RotateCcw } from 'lucide-react'

interface ReviewItem {
  target_type: 'word' | 'phrase'
  id: number
  display: string
  familiarity: number
  interval: number
  ease_factor: number
}

interface ReviewTodayResponse {
  items: ReviewItem[]
}

export default function Review() {
  const [flipped, setFlipped] = useState(false)
  const [submitted, setSubmitted] = useState<number[]>([])
  const [stats, setStats] = useState({ correct: 0, wrong: 0 })

  const { data: resp, isLoading } = useQuery<ReviewTodayResponse>({
    queryKey: ['review-today'],
    queryFn: () => apiFetch<ReviewTodayResponse>('/review/today'),
  })

  const queue = resp?.items ?? []
  const remaining = queue.filter(item => !submitted.includes(item.id))
  const current = remaining.length > 0 ? remaining[0] : null
  const progress = submitted.length
  const total = queue.length

  const handleRate = async (quality: number) => {
    if (!current) return
    try {
      await apiFetch(`/review/${current.target_type}/${current.id}`, {
        method: 'POST',
        body: JSON.stringify({ quality }),
      })
      setSubmitted(prev => [...prev, current.id])
      if (quality >= 3) setStats(s => ({ ...s, correct: s.correct + 1 }))
      else setStats(s => ({ ...s, wrong: s.wrong + 1 }))
      setFlipped(false)
    } catch {
      console.error('Failed to submit review')
    }
  }

  const handleReset = () => {
    setFlipped(false)
    setSubmitted([])
    setStats({ correct: 0, wrong: 0 })
  }

  if (isLoading) {
    return (
      <div className="p-6 flex flex-col items-center justify-center">
        <div className="h-64 w-full max-w-lg bg-muted animate-pulse rounded-xl" />
      </div>
    )
  }

  if (!current) {
    return (
      <div className="p-6 flex flex-col items-center justify-center gap-4">
        <h2 className="text-2xl font-bold">
          {total === 0 ? 'No reviews due!' : 'Review complete!'}
        </h2>
        {total > 0 && (
          <div className="text-muted-foreground">
            {stats.correct} correct, {stats.wrong} need review
          </div>
        )}
        {total > 0 && (
          <Button variant="outline" onClick={handleReset}>
            <RotateCcw className="h-4 w-4 mr-1" />Restart
          </Button>
        )}
      </div>
    )
  }

  const qualityLabels = ['Again', 'Hard', 'Difficult', 'Good', 'Easy', 'Perfect']
  const qualityColors = [
    'bg-red-500 hover:bg-red-600',
    'bg-orange-500 hover:bg-orange-600',
    'bg-amber-500 hover:bg-amber-600',
    'bg-green-500 hover:bg-green-600',
    'bg-emerald-500 hover:bg-emerald-600',
    'bg-blue-500 hover:bg-blue-600',
  ]

  return (
    <div className="p-6 flex flex-col items-center gap-6 max-w-2xl mx-auto">
      <div className="w-full flex items-center justify-between">
        <h1 className="text-2xl font-bold">Review</h1>
        <span className="text-sm text-muted-foreground">
          {progress} / {total}
        </span>
      </div>
      <div className="w-full bg-muted rounded-full h-2">
        <div
          className="bg-primary h-2 rounded-full transition-all"
          style={{ width: total > 0 ? `${(progress / total) * 100}%` : '0%' }}
        />
      </div>

      <div
        className="w-full cursor-pointer min-h-[200px]"
        onClick={() => setFlipped(!flipped)}
      >
        <Card className="w-full min-h-[200px] flex items-center justify-center transition-all">
          <CardContent className="text-center p-8">
            {!flipped ? (
              <div className="text-3xl font-bold">{current.display}</div>
            ) : (
              <div>
                <div className="text-3xl font-bold mb-2">{current.display}</div>
                <div className="text-sm text-muted-foreground">
                  Familiarity: Lv.{current.familiarity} · Interval: {current.interval}d · Ease: {current.ease_factor.toFixed(2)}
                </div>
                <div className="text-xs text-muted-foreground mt-2 capitalize">
                  {current.target_type}
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {!flipped ? (
        <p className="text-sm text-muted-foreground">Click the card to reveal the answer</p>
      ) : (
        <div className="flex gap-2 flex-wrap justify-center">
          {qualityLabels.map((label, i) => (
            <Button
              key={label}
              className={`${qualityColors[i]} text-white`}
              onClick={() => handleRate(i)}
            >
              {label}
            </Button>
          ))}
        </div>
      )}
    </div>
  )
}
