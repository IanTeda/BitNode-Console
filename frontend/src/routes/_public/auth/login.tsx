import { createFileRoute } from '@tanstack/react-router'
import logger from '@/lib/logger'

const log = logger.getSubLogger({ name: 'LoginRoute' })

export const Route = createFileRoute('/_public/auth/login')({
  component: RouteComponent,
})

function RouteComponent() {
  log.info('Login page rendered')
  return <div>Hello "/_public/auth/login"!</div>
}
