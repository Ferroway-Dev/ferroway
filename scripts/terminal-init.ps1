# Ferroway workspace terminal initializer.
# Loaded automatically by the "Ferroway Shell" terminal profile in ferroway.code-workspace.
# Switches between pipeline/.venv, forge/.venv, and the bare Rust environment
# whenever you cd between subprojects.

$script:FerrRoot    = Split-Path -Parent $PSScriptRoot
$script:FerrActive  = $null   # tracks which env is currently live

function _Ferroway_SwitchEnv {
    $here = (Get-Location).Path

    # Classify the current directory.
    if ($here.StartsWith((Join-Path $script:FerrRoot 'pipeline'))) { $next = 'pipeline' }
    elseif ($here.StartsWith((Join-Path $script:FerrRoot 'forge'))) { $next = 'forge' }
    elseif ($here.StartsWith((Join-Path $script:FerrRoot 'simulator'))) { $next = 'simulator' }
    else { $next = 'root' }

    if ($next -eq $script:FerrActive) { return }   # already correct — no-op

    # Deactivate any active Python venv before switching.
    if (Get-Command deactivate -ErrorAction SilentlyContinue) { deactivate }

    switch ($next) {
        'pipeline' {
            $activate = Join-Path $script:FerrRoot 'pipeline\.venv\Scripts\Activate.ps1'
            if (Test-Path $activate) { & $activate }
        }
        'forge' {
            $activate = Join-Path $script:FerrRoot 'forge\.venv\Scripts\Activate.ps1'
            if (Test-Path $activate) { & $activate }
        }
        # 'simulator' / 'root': Rust is on PATH globally via rustup — nothing to activate.
    }

    $script:FerrActive = $next
}

# ── Prompt hook ───────────────────────────────────────────────────────────────
# Chain onto whatever prompt was already defined (oh-my-posh, starship, etc.)
# so we don't break a custom prompt.

$script:FerrWrappedPrompt = $function:prompt   # capture original (may be $null)
$script:FerrLastDir        = $null

function global:prompt {
    $here = (Get-Location).Path
    if ($here -ne $script:FerrLastDir) {
        $script:FerrLastDir = $here
        _Ferroway_SwitchEnv
    }

    # Delegate to the original prompt if one exists; otherwise use the PS default.
    if ($script:FerrWrappedPrompt) {
        & $script:FerrWrappedPrompt
    } else {
        "PS $here$('>' * ($nestedPromptLevel + 1)) "
    }
}

# ── Activate immediately for the shell's initial directory ────────────────────
_Ferroway_SwitchEnv
