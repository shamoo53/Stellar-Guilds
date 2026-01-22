import Link from 'next/link'

export default function NotFound() {
  return (
    <div className="flex flex-col items-center justify-center min-h-screen bg-stellar-navy px-4">
      <div className="text-center max-w-md">
        <div className="w-24 h-24 rounded-full bg-stellar-lightNavy flex items-center justify-center mx-auto mb-6">
          <span className="text-4xl text-stellar-slate">404</span>
        </div>
        
        <h1 className="text-3xl font-bold text-stellar-white mb-4">
          Page Not Found
        </h1>
        
        <p className="text-stellar-slate mb-8">
          The page you&apos;re looking for doesn&apos;t exist or has been moved.
        </p>
        
        <Link 
          href="/"
          className="inline-flex items-center px-6 py-3 bg-gradient-to-r from-gold-500 to-gold-600 text-stellar-navy font-semibold rounded-lg hover:from-gold-400 hover:to-gold-500 transition-all"
        >
          Return Home
        </Link>
      </div>
    </div>
  )
}