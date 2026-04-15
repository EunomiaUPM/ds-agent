import { createFileRoute, Outlet } from '@tanstack/react-router';

export const Route = createFileRoute('/my-catalog')({
  component: RouteComponent,
})

function RouteComponent() {
  return (
    <Outlet/>
  )
}
