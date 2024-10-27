import { rspc } from '@/lib/rspc'
import Icon from '@gendam/ui/icons'
import { useMemo, useState } from 'react'

export default function SearchSuggestions({ onSelectText }: { onSelectText: (text: string) => void }) {
  const suggestionsQuery = rspc.useQuery(['search.suggestions'])
  const [suggestSeed, setSuggestSeed] = useState(0)
  const pickedSuggestions = useMemo(() => {
    // shuffle pick 5 suggestions
    suggestSeed
    if (suggestionsQuery.data) {
      const suggestions = [...suggestionsQuery.data]
      const picked = []
      while (picked.length < 5 && suggestions.length > 0) {
        const index = Math.floor(Math.random() * suggestions.length)
        picked.push(suggestions[index])
        suggestions.splice(index, 1)
      }
      return picked
    } else {
      return []
    }
  }, [suggestionsQuery.data, suggestSeed])

  return (
    <>
      <div className="text-ink/50 mb-2 text-xs">
        {pickedSuggestions.map((suggestion, index) => (
          <div key={index} className="py-1 text-center hover:underline" onClick={() => onSelectText(suggestion)}>
            &quot;{suggestion}&quot;
          </div>
        ))}
      </div>
      <div className="mb-4 p-2" onClick={() => setSuggestSeed(suggestSeed + 1)}>
        <Icon.Cycle className="text-ink/50 h-4 w-4" />
      </div>
    </>
  )
}
