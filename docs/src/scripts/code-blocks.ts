// Setup code block copy functionality
function setupCodeBlocks(): void {
    document.querySelectorAll<HTMLButtonElement>('.copy-button').forEach((button) => {
        button.addEventListener('click', async () => {
            const code = button.dataset.code;
            if (!code) return;

            try {
                await navigator.clipboard.writeText(code);
                
                // Update button UI
                const copyIcon = button.querySelector('.copy-icon');
                const checkIcon = button.querySelector('.check-icon');
                const copyText = button.querySelector('.copy-text');
                const copiedText = button.querySelector('.copied-text');
                
                copyIcon?.classList.add('hidden');
                checkIcon?.classList.remove('hidden');
                copyText?.classList.add('hidden');
                copiedText?.classList.remove('hidden');
                
                // Reset after 2 seconds
                setTimeout(() => {
                    copyIcon?.classList.remove('hidden');
                    checkIcon?.classList.add('hidden');
                    copyText?.classList.remove('hidden');
                    copiedText?.classList.add('hidden');
                }, 2000);
            } catch (err) {
                console.error('Failed to copy code:', err);
            }
        });
    });
}

// Run on load and after navigation
setupCodeBlocks();
document.addEventListener('astro:after-swap', setupCodeBlocks);
