import { DatabaseService } from './src/services/database.js'

async function testDatabase() {
  try {
    console.log('Testing database health check...')
    const health = await DatabaseService.healthCheck()
    console.log('Database health:', health)

    console.log('\nTesting repository creation...')
    const repo = await DatabaseService.createRepository({
      name: 'Test Repository',
      path: '/Users/test/project',
      source_branch: 'main',
      init_script: 'npm install'
    })
    console.log('Created repository:', repo)

    console.log('\nTesting get repositories...')
    const repos = await DatabaseService.getRepositories()
    console.log('All repositories:', repos)

    console.log('\nTesting workspace creation...')
    const workspace = await DatabaseService.createWorkspace({
      repository_id: repo.id,
      name: 'feature-test',
      branch: 'feature/test'
    }, '/Users/test/project/.workhorse/feature-test')
    console.log('Created workspace:', workspace)

    console.log('\nTesting get workspaces...')
    const workspaces = await DatabaseService.getWorkspacesByRepository(repo.id)
    console.log('Repository workspaces:', workspaces)

    console.log('\n✅ All database tests passed!')
  } catch (error) {
    console.error('❌ Database test failed:', error)
  }
}

// Note: This would be run in the browser console or in a test environment
// testDatabase()

export { testDatabase }