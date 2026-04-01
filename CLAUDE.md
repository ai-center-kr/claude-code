# Claude Code Internals

Claude Code 소스 유출 분석 + Clean-room Rust 재구현 프로젝트.

## Overview
- 2026-03-31 npm sourcemap 유출로 드러난 Claude Code 내부 구조 분석
- 행동 스펙 기반의 독립적인 Rust 재구현 (Phoenix v. IBM 클린룸 방식)

## Directory Structure #260401-11
```
README.md              # 유출 분석 상세 breakdown
spec/                  # 15개 행동 스펙 문서 (~990KB)
  00_overview.md       # 마스터 아키텍처
  01-13_*.md           # 코어, 명령어, 도구, 컴포넌트, 서비스, 훅 등
  INDEX.md             # 스펙 인덱스 + Quick Lookup
src-rust/              # Rust 재구현 (10 crates workspace)
  Cargo.toml           # Workspace root
  crates/
    core/              # 핵심 타입, 설정, 시스템 프롬프트, 분석
    api/               # Anthropic API 클라이언트, SSE 스트리밍
    tools/             # 33개 도구 구현 (bash, file, grep, agent 등)
    query/             # 쿼리 루프, 에이전트, autoDream, coordinator
    tui/               # ratatui 기반 터미널 UI
    commands/          # 35+ 슬래시 명령어
    mcp/               # MCP 프로토콜 클라이언트
    bridge/            # JWT 인증, 원격 브리지 세션
    cli/               # 바이너리 진입점, OAuth 플로우
    buddy/             # 타마고치 컴패니언 시스템
docs/                  # Mintlify 문서 사이트 (41 MDX 페이지)
  docs.json            # 네비게이션 설정 (maple 테마)
  overview/            # 소개, 유출 경위, 핵심 발견
  deep-dive/           # BUDDY, KAIROS, ULTRAPLAN, Dream 등 상세
  spec/                # 스펙 문서 요약 페이지
  rust-impl/           # Rust 크레이트별 문서
public/                # 이미지 (leak-tweet.png, claude-files.png)
```

## Dev Commands #260401-11
```bash
# Rust 빌드
cd src-rust && cargo build

# Mintlify 문서 로컬 서버
cd docs && PATH="$HOME/.nvm/versions/node/v20.18.0/bin:$PATH" npx mintlify dev --port 3030
```

## Key Numbers
- TypeScript 원본: ~1,902 파일, 800K+ LoC
- 스펙: 15 문서, ~990KB
- Rust 크레이트: 10개
- 도구: 40+ (원본) / 33 (Rust)
- 슬래시 명령어: 100+ (원본) / 35+ (Rust)
- 문서 페이지: 41 MDX
