import { create } from 'zustand'
import { ErrorHandler } from '../utils/errorHandler'
import { storeService } from '../services/storeService'

export interface Article {
  id: string
  title: string
  content: string
  status: 'draft' | 'published'
  createdAt: string
  updatedAt: string
  qualityReport?: string | null
}

interface ArticleStore {
  articles: Article[]
  currentArticle: Article | null
  isLoading: boolean
  setArticles: (articles: Article[]) => void
  setCurrentArticle: (article: Article | null) => void
  addArticle: (article: Omit<Article, 'id' | 'createdAt' | 'updatedAt'>) => Promise<void>
  updateArticle: (id: string, updates: Partial<Article>) => Promise<void>
  deleteArticle: (id: string) => Promise<void>
  loadArticles: () => Promise<void>
  persistArticles: () => Promise<void>
}

export const useArticleStore = create<ArticleStore>((set, get) => ({
  articles: [],
  currentArticle: null,
  isLoading: false,

  setArticles: (articles) => set({ articles }),

  setCurrentArticle: (article) => set({ currentArticle: article }),

  addArticle: async (article) => {
    const now = new Date().toISOString()
    const newArticle: Article = {
      ...article,
      id: Date.now().toString(),
      status: 'draft',
      createdAt: now,
      updatedAt: now,
    }
    set((state) => {
      const newArticles = [...state.articles, newArticle]
      return { articles: newArticles, currentArticle: newArticle }
    })
    await get().persistArticles()
  },

  updateArticle: async (id, updates) => {
    set((state) => {
      const newArticles = state.articles.map((article) =>
        article.id === id
          ? { ...article, ...updates, updatedAt: new Date().toISOString() }
          : article
      )
      const newCurrentArticle =
        state.currentArticle?.id === id
          ? { ...state.currentArticle, ...updates, updatedAt: new Date().toISOString() }
          : state.currentArticle
      return { articles: newArticles, currentArticle: newCurrentArticle }
    })
    await get().persistArticles()
  },

  deleteArticle: async (id) => {
    try {
      await storeService.deleteArticle(id)
    } catch (e) {
      console.error('删除文章失败:', e)
    }
    const newArticles = get().articles.filter((article) => article.id !== id)
    const newCurrentArticle = get().currentArticle?.id === id ? null : get().currentArticle
    set({ articles: newArticles, currentArticle: newCurrentArticle })
    await get().persistArticles()
  },

  loadArticles: async () => {
    set({ isLoading: true })
    try {
      const articles = await storeService.getArticles()
      const processedArticles = articles.map((article: any) => ({
        id: article.id,
        title: article.title,
        content: article.content,
        status: (article.status ?? 'draft') as 'draft' | 'published',
        createdAt: article.created_at || article.createdAt || new Date().toISOString(),
        updatedAt: article.updated_at || article.updatedAt || new Date().toISOString(),
        qualityReport: article.quality_report ?? null,
      }))
      set({ articles: processedArticles, isLoading: false })
    } catch (error) {
      ErrorHandler.handleError(error, {
        message: '加载文章失败'
      })
      set({ isLoading: false })
    }
  },

  persistArticles: (() => {
    let debounceTimer: NodeJS.Timeout | null = null
    return async () => {
      if (debounceTimer) {
        clearTimeout(debounceTimer)
      }

      debounceTimer = setTimeout(async () => {
        try {
          await storeService.saveArticles(get().articles as any)
        } catch (error) {
          ErrorHandler.handleError(error, {
            message: '保存文章失败'
          })
        } finally {
          debounceTimer = null
        }
      }, 500)
    }
  })(),
}))
