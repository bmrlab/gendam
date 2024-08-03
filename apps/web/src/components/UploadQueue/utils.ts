import { SUPPORTED_CONTENT_TYPES } from '@/constants'

export const filterFiles = (files: string[]) => {
  const supportedFiles: string[] = []
  const unsupportedExtensionsSet: Set<string> = new Set()
  // 过滤
  files.forEach((file) => {
    const extension = file.split('.').pop()?.toLowerCase()
    if (extension) {
      if (SUPPORTED_CONTENT_TYPES.has(extension)) {
        supportedFiles.push(file)
      } else {
        unsupportedExtensionsSet.add(extension)
      }
    }
  })
  return {
    supportedFiles,
    unsupportedExtensionsSet,
  }
}
