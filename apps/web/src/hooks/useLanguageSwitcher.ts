import { SupportedLanguage } from '@/constants'
import { useTranslation } from 'react-i18next'

const useLanguageSwitcher = () => {
  const { i18n } = useTranslation()

  const changeLanguage = (lng: SupportedLanguage) => {
    i18n.changeLanguage(lng)
    localStorage.setItem("lng", lng)
  }

  return { changeLanguage, currentLanguage: i18n.language }
}

export default useLanguageSwitcher
