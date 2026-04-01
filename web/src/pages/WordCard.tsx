import { useState } from 'react'
import { useParams, Link } from '@tanstack/react-router'
import { useWord } from '@/features/vocab/queries'
import { fetchSSE } from '@/lib/api-client'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Skeleton } from '@/components/ui/skeleton'
import FamiliarityBadge from '@/components/FamiliarityBadge'
import MarkdownRender from '@/components/MarkdownRender'
import { Sparkles, ArrowLeft } from 'lucide-react'

export default function WordCard() {
  const { word } = useParams({ from: '/words/$word' })
  const [explanation, setExplanation] = useState('')
  const [isStreaming, setIsStreaming] = useState(false)
  const [streamError, setStreamError] = useState('')

  const { data, isLoading } = useWord(word)

  const handleExplain = () => {
    if (!word || isStreaming) return
    setExplanation('')
    setStreamError('')
    setIsStreaming(true)
    fetchSSE(
      `/words/${encodeURIComponent(word)}/explain`,
      (chunk) => setExplanation(prev => prev + chunk),
      () => setIsStreaming(false),
      (err) => {
        setStreamError(err.message)
        setIsStreaming(false)
      },
    )
  }

  if (isLoading) {
    return (
      <div className="p-6 space-y-4">
        <Skeleton className="h-8 w-48" />
        <Skeleton className="h-4 w-32" />
        <Skeleton className="h-24 w-full" />
        <Skeleton className="h-32 w-full" />
      </div>
    )
  }

  if (!data) {
    return (
      <div className="p-6">
        <p className="text-muted-foreground">Word not found</p>
        <Link to="/vocabulary">
          <Button variant="link"><ArrowLeft className="h-4 w-4 mr-1" />Back to Vocabulary</Button>
        </Link>
      </div>
    )
  }

  return (
    <div className="p-6 max-w-3xl mx-auto">
      <Link to="/vocabulary">
        <Button variant="ghost" size="sm" className="mb-4">
          <ArrowLeft className="h-4 w-4 mr-1" />Back
        </Button>
      </Link>
      <Card className="mb-6">
        <CardHeader>
          <div className="flex items-center gap-3">
            <CardTitle className="text-3xl">{data.word}</CardTitle>
            <FamiliarityBadge level={data.familiarity} />
          </div>
          {data.phonetic && (
            <Badge variant="outline" className="w-fit">{data.phonetic}</Badge>
          )}
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <h3 className="text-sm font-medium text-muted-foreground mb-1">Meaning</h3>
            <p className="text-lg">{data.meaning}</p>
          </div>
          {data.examples && data.examples.length > 0 && (
            <div>
              <h3 className="text-sm font-medium text-muted-foreground mb-2">Examples</h3>
              <ul className="space-y-1">
                {data.examples.map((ex, i) => (
                  <li key={i} className="text-sm italic text-muted-foreground">• {ex}</li>
                ))}
              </ul>
            </div>
          )}
          {data.notes && (
            <div>
              <h3 className="text-sm font-medium text-muted-foreground mb-1">Notes</h3>
              <p className="text-sm">{data.notes}</p>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="text-lg">AI Explanation</CardTitle>
            <Button onClick={handleExplain} disabled={isStreaming} size="sm">
              <Sparkles className="h-4 w-4 mr-1" />
              {isStreaming ? 'Explaining...' : 'Explain'}
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {streamError && <p className="text-sm text-destructive">{streamError}</p>}
          {explanation || isStreaming ? (
            <ScrollArea className="max-h-96">
              <div className="prose prose-sm max-w-none">
                <MarkdownRender content={explanation} />
                {isStreaming && <span className="animate-pulse">▊</span>}
              </div>
            </ScrollArea>
          ) : (
            <p className="text-sm text-muted-foreground">
              Click "Explain" to get an AI explanation of this word.
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
