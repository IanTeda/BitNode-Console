import { createFileRoute } from '@tanstack/react-router'
import logger from '@/lib/logger'

const log = logger.getSubLogger({ name: 'LogoutRoute' })

export const Route = createFileRoute('/_public/auth/logout')({
  component: RouteComponent,
})

function RouteComponent() {
  log.info('Logout page rendered')
  return <div>Hello "/_public/auth/logout"!</div>
}
