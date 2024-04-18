import { rspc } from '@/lib/rspc'
import { useEffect } from 'react'
import { ModelItem } from './ModelItem'

export function ModelList() {
  const { data } = rspc.useQuery(['libraries.models.list'])
  const { data: settings } = rspc.useQuery(['libraries.get_library_settings'])

  useEffect(() => {
    console.log(data)
  }, [data])

  useEffect(() => {
    console.log(settings)
  }, [settings])

  return (
    <div>
      {settings &&
        (['MultiModalEmbedding', 'TextEmbedding', 'ImageCaption', 'AudioTranscript'] as const).map((category) => (
          <div key={category} className="mt-4">
            <div className="mb-2 text-lg font-bold">{category}</div>
            <div>
              {data
                ?.find((v) => v.category === category)
                ?.models.map((t) => (
                  <ModelItem
                    key={t.info.id}
                    model={t}
                    category={category}
                    activated={t.info.id === settings.models[category]}
                  />
                ))}
            </div>
          </div>
        ))}
    </div>
  )
}
