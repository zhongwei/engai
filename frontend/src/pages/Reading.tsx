import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { apiFetch, fetchSSE } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Skeleton } from '@/components/ui/skeleton'
import MarkdownRender from '@/components/MarkdownRender'
import { Sparkles, FileText } from 'lucide-react'

interface Reading {
  id: number
  title: string
  content?: string
  source?: string
  created_at: string
}

interface ReadingDetail {
  id: number
  title: string
  content: string
  source?: string
  created_at: string
}

interface ReadingsResponse {
  readings: Reading[]
}

export default function Reading() {
  const [selectedId, setSelectedId] = useState<number | null>(null)
  const [analysis, setAnalysis] = useState('')
  const [isStreaming, setIsStreaming] = useState(false)

  const { data: readingsResp, isLoading } = useQuery<ReadingsResponse>({
    queryKey: ['readings'],
    queryFn: () => apiFetch<ReadingsResponse>('/readings'),
  })

  const readings = readingsResp?.readings ?? []

  const { data: detail } = useQuery<ReadingDetail>({
    queryKey: ['reading', selectedId],
    queryFn: () => apiFetch<ReadingDetail>(`/readings/${selectedId}`),
    enabled: !!selectedId,
  })

  const selected = selectedId ? readings.find(r => r.id === selectedId) : null

  const handleAnalyze = () => {
    if (!selectedId || isStreaming) return
    setAnalysis('')
    setIsStreaming(true)
    fetchSSE(
      `/readings/${selectedId}/analyze`,
      (chunk) => setAnalysis(prev => prev + chunk),
      () => setIsStreaming(false),
      () => setIsStreaming(false),
    )
  }

  return (
    <div className="p-6 flex gap-4 h-full">
      <div className="w-64 shrink-0 border-r pr-4">
        <h2 className="text-lg font-bold mb-3">Readings</h2>
        {isLoading ? (
          <div className="space-y-2">
            {[...Array(5)].map((_, i) => (
              <Skeleton key={i} className="h-12 w-full" />
            ))}
          </div>
        ) : readings.length === 0 ? (
          <p className="text-sm text-muted-foreground">No readings yet</p>
        ) : (
          <ScrollArea className="h-[calc(100vh-120px)]">
            <div className="space-y-1">
              {readings.map(r => (
                <button
                  key={r.id}
                  onClick={() => { setSelectedId(r.id); setAnalysis('') }}
                  className={`w-full text-left px-3 py-2 rounded-md text-sm transition-colors ${
                    selectedId === r.id
                      ? 'bg-accent text-accent-foreground'
                      : 'hover:bg-muted'
                  }`}
                >
                  <div className="font-medium truncate">{r.title}</div>
                  <div className="text-xs text-muted-foreground">
                    {new Date(r.created_at).toLocaleDateString()}
                  </div>
                </button>
              ))}
            </div>
          </ScrollArea>
        )}
      </div>

      <div className="flex-1 min-w-0">
        {selected ? (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-2xl font-bold">{detail?.title || selected.title}</h1>
                {detail?.source && (
                  <p className="text-sm text-muted-foreground">{detail.source}</p>
                )}
              </div>
              <Button onClick={handleAnalyze} disabled={isStreaming} size="sm">
                <Sparkles className="h-4 w-4 mr-1" />
                {isStreaming ? 'Analyzing...' : 'AI Analysis'}
              </Button>
            </div>

            <Card>
              <CardContent className="p-4">
                <ScrollArea className="max-h-[30vh]">
                  <p className="text-sm leading-relaxed whitespace-pre-wrap">
                    {detail?.content || selected.content || 'Loading...'}
                  </p>
                </ScrollArea>
              </CardContent>
            </Card>

            {analysis && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">AI Analysis</CardTitle>
                </CardHeader>
                <CardContent>
                  <ScrollArea className="max-h-[30vh]">
                    <MarkdownRender content={analysis} />
                    {isStreaming && <span className="animate-pulse">▊</span>}
                  </ScrollArea>
                </CardContent>
              </Card>
            )}
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
            <FileText className="h-12 w-12 mb-3 opacity-50" />
            <p>Select a reading to view its content</p>
          </div>
        )}
      </div>
    </div>
  )
}
