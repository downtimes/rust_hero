let SessionLoad = 1
if &cp | set nocp | endif
let s:cpo_save=&cpo
set cpo&vim
cmap <S-Insert> +
inoremap <C-Tab> w
cnoremap <C-Tab> w
inoremap <C-F4> c
cnoremap <C-F4> c
inoremap <C-Space> 
imap <Nul> <C-Space>
inoremap <expr> <Up> pumvisible() ? "\" : "\<Up>"
inoremap <expr> <S-Tab> pumvisible() ? "\" : "\<S-Tab>"
inoremap <expr> <Down> pumvisible() ? "\" : "\<Down>"
imap <S-Insert> 
nnoremap  gggHG
onoremap  gggHG
snoremap  gggHG
xnoremap  ggVG
vnoremap  "+y
nnoremap  h
nnoremap <NL> j
nnoremap  k
nnoremap  l
noremap  
onoremap  :update
nnoremap  :update
vnoremap  :update
omap  "+gP
nmap  "+gP
vnoremap  "+x
noremap  
noremap  u
inoremap   :simalt ~
cnoremap   :simalt ~
map Q gq
vmap [% [%m'gv``
vmap ]% ]%m'gv``
vmap a% [%v]%
vmap gx <Plug>NetrwBrowseXVis
nmap gx <Plug>NetrwBrowseX
nnoremap gd :YcmCompleter GoTo
omap <S-Insert> "+gP
vnoremap <BS> d
vnoremap <C-Tab> w
nnoremap <C-Tab> w
onoremap <C-Tab> w
vnoremap <C-F4> c
nnoremap <C-F4> c
onoremap <C-F4> c
vnoremap <silent> <Plug>NetrwBrowseXVis :call netrw#BrowseXVis()
nnoremap <silent> <Plug>NetrwBrowseX :call netrw#NetrwBrowseX(expand("<cWORD>"),0)
vmap <C-Del> "*d
vnoremap <S-Del> "+x
vnoremap <C-Insert> "+y
vmap <S-Insert> 
nmap <S-Insert> "+gP
inoremap  gggHG
cnoremap  gggHG
inoremap  h
inoremap 	 
inoremap <NL> j
inoremap  k
inoremap  l
inoremap  :update
inoremap  u
cmap  +
inoremap  
inoremap  u
noremap   :simalt ~
nnoremap Ã¶d :YcmShowDetailedDiagnostic
nnoremap Ã¶r :make run
nnoremap Ã¶b :make
inoremap jk  
let &cpo=s:cpo_save
unlet s:cpo_save
set autowrite
set background=dark
set backspace=indent,eol,start
set backup
set completefunc=youcompleteme#Complete
set completeopt=preview,menuone
set cpoptions=aAceFsB
set diffexpr=MyDiff()
set encoding=utf-8
set expandtab
set fileencodings=ucs-bom,utf-8,default,latin1
set guifont=Liberation_Mono:h10:cANSI
set helplang=de
set hidden
set hlsearch
set ignorecase
set incsearch
set keymodel=startsel,stopsel
set makeprg=build.bat
set matchpairs=(:),{:},[:],<:>
set ruler
set runtimepath=~/vimfiles,~\\vimfiles\\bundle\\Vundle.vim,~\\vimfiles\\bundle\\rust.vim,~\\vimfiles\\bundle\\molokai,~\\vimfiles\\bundle\\vim-racer,C:\\Program\ Files\ (x86)\\Vim/vimfiles,C:\\Program\ Files\ (x86)\\Vim\\vim74,C:\\Program\ Files\ (x86)\\Vim/vimfiles/after,~/vimfiles/after,~/vimfiles/bundle/Vundle.vim,~\\vimfiles\\bundle\\Vundle.vim/after,~\\vimfiles\\bundle\\rust.vim/after,~\\vimfiles\\bundle\\molokai/after,~\\vimfiles\\bundle\\vim-racer/after,C:/Program\ Files/Vim/vim74/pack/dist/opt/matchit
set scrolljump=5
set scrolloff=10
set selection=exclusive
set selectmode=mouse,key
set shellslash
set shiftwidth=4
set smartcase
set smarttab
set updatetime=2000
set window=61
let s:so_save = &so | let s:siso_save = &siso | set so=0 siso=0
let v:this_session=expand("<sfile>:p")
silent only
cd D:\WinWorkspace\rust_hero
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
set shortmess=aoO
badd +33 Session.vim
badd +9 src/main.rs
badd +178 src/common/mod.rs
badd +1 src/ffi/direct_sound.rs
badd +1 src/ffi/linux.rs
badd +667 src/ffi/mod.rs
badd +1 src/ffi/sdl.rs
badd +1 src/game/entity.rs
badd +1 src/game/graphics.rs
badd +1 src/game/math.rs
badd +1 src/game/memory.rs
badd +584 src/game/mod.rs
badd +1 src/game/random.rs
badd +305 src/game/simulation.rs
badd +1 src/game/world.rs
badd +1 src/libgame.rs
badd +1 src/linux.rs
badd +1405 src/win32.rs
argglobal
silent! argdel *
argadd src/common/mod.rs
argadd src/ffi/direct_sound.rs
argadd src/ffi/linux.rs
argadd src/ffi/mod.rs
argadd src/ffi/sdl.rs
argadd src/game/entity.rs
argadd src/game/graphics.rs
argadd src/game/math.rs
argadd src/game/memory.rs
argadd src/game/mod.rs
argadd src/game/random.rs
argadd src/game/simulation.rs
argadd src/game/world.rs
argadd src/libgame.rs
argadd src/linux.rs
argadd src/main.rs
argadd src/win32.rs
edit src/game/world.rs
set splitbelow splitright
wincmd _ | wincmd |
vsplit
1wincmd h
wincmd w
wincmd _ | wincmd |
split
1wincmd k
wincmd w
set nosplitbelow
set nosplitright
wincmd t
set winheight=1 winwidth=1
exe 'vert 1resize ' . ((&columns * 167 + 117) / 235)
exe '2resize ' . ((&lines * 53 + 31) / 62)
exe 'vert 2resize ' . ((&columns * 67 + 117) / 235)
exe '3resize ' . ((&lines * 6 + 31) / 62)
exe 'vert 3resize ' . ((&columns * 67 + 117) / 235)
argglobal
edit src/game/world.rs
nnoremap <buffer> <silent> <D-r> :RustRun
nnoremap <buffer> <D-R> :RustRun! =join(b:rust_last_rustc_args)erust#AppendCmdLine(' -- ' . join(b:rust_last_args))
nnoremap <buffer> <silent> [[ :call rust#Jump('n', 'Back')
xnoremap <buffer> <silent> [[ :call rust#Jump('v', 'Back')
onoremap <buffer> <silent> [[ :call rust#Jump('o', 'Back')
nnoremap <buffer> <silent> ]] :call rust#Jump('n', 'Forward')
xnoremap <buffer> <silent> ]] :call rust#Jump('v', 'Forward')
onoremap <buffer> <silent> ]] :call rust#Jump('o', 'Forward')
setlocal keymap=
setlocal noarabic
setlocal autoindent
setlocal backupcopy=
setlocal balloonexpr=
setlocal nobinary
setlocal nobreakindent
setlocal breakindentopt=
setlocal bufhidden=
setlocal buflisted
setlocal buftype=
setlocal cindent
setlocal cinkeys=0{,0},!^F,o,O,0[,0]
setlocal cinoptions=L0,(0,Ws,J1,j1
setlocal cinwords=for,if,else,while,loop,impl,mod,unsafe,trait,struct,enum,fn,extern
set colorcolumn=80
setlocal colorcolumn=80
setlocal comments=s0:/*!,m:\ ,ex:*/,s1:/*,mb:*,ex:*/,:///,://!,://
setlocal commentstring=//%s
setlocal complete=.,w,b,u,t,i
setlocal concealcursor=
setlocal conceallevel=0
setlocal completefunc=youcompleteme#Complete
setlocal nocopyindent
setlocal cryptmethod=
setlocal nocursorbind
setlocal nocursorcolumn
setlocal nocursorline
setlocal define=
setlocal dictionary=
setlocal nodiff
setlocal equalprg=
setlocal errorformat=
setlocal expandtab
if &filetype != 'rust'
setlocal filetype=rust
endif
setlocal fixendofline
setlocal foldcolumn=0
setlocal foldenable
setlocal foldexpr=0
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldmarker={{{,}}}
setlocal foldmethod=manual
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldtext=foldtext()
setlocal formatexpr=
setlocal formatoptions=croqnlj
setlocal formatlistpat=^\\s*\\d\\+[\\]:.)}\\t\ ]\\s*
setlocal grepprg=
setlocal iminsert=0
setlocal imsearch=0
setlocal include=
setlocal includeexpr=substitute(v:fname,'::','/','g')
setlocal indentexpr=GetRustIndent(v:lnum)
setlocal indentkeys=0{,0},!^F,o,O,0[,0]
setlocal noinfercase
setlocal iskeyword=@,48-57,_,192-255
setlocal keywordprg=
setlocal nolinebreak
setlocal nolisp
setlocal lispwords=
setlocal nolist
setlocal makeprg=
setlocal matchpairs=(:),{:},[:],<:>
setlocal modeline
setlocal modifiable
setlocal nrformats=octal,hex
set number
setlocal number
setlocal numberwidth=4
setlocal omnifunc=RacerComplete
setlocal path=
setlocal nopreserveindent
setlocal nopreviewwindow
setlocal quoteescape=\\
setlocal noreadonly
setlocal norelativenumber
setlocal norightleft
setlocal rightleftcmd=search
setlocal noscrollbind
setlocal shiftwidth=4
setlocal noshortname
setlocal smartindent
setlocal softtabstop=4
setlocal nospell
setlocal spellcapcheck=[.?!]\\_[\\])'\"\	\ ]\\+
setlocal spellfile=
setlocal spelllang=en
setlocal statusline=
setlocal suffixesadd=.rs
setlocal swapfile
setlocal synmaxcol=3000
if &syntax != 'rust'
setlocal syntax=rust
endif
setlocal tabstop=4
setlocal tagcase=
setlocal tags=
setlocal textwidth=99
setlocal thesaurus=
setlocal noundofile
setlocal undolevels=-123456
setlocal nowinfixheight
setlocal nowinfixwidth
setlocal wrap
setlocal wrapmargin=0
silent! normal! zE
let s:l = 402 - ((26 * winheight(0) + 30) / 60)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
402
normal! 0
wincmd w
argglobal
edit src/common/mod.rs
nnoremap <buffer> <silent> <D-r> :RustRun
nnoremap <buffer> <D-R> :RustRun! =join(b:rust_last_rustc_args)erust#AppendCmdLine(' -- ' . join(b:rust_last_args))
nnoremap <buffer> <silent> [[ :call rust#Jump('n', 'Back')
xnoremap <buffer> <silent> [[ :call rust#Jump('v', 'Back')
onoremap <buffer> <silent> [[ :call rust#Jump('o', 'Back')
nnoremap <buffer> <silent> ]] :call rust#Jump('n', 'Forward')
xnoremap <buffer> <silent> ]] :call rust#Jump('v', 'Forward')
onoremap <buffer> <silent> ]] :call rust#Jump('o', 'Forward')
setlocal keymap=
setlocal noarabic
setlocal autoindent
setlocal backupcopy=
setlocal balloonexpr=
setlocal nobinary
setlocal nobreakindent
setlocal breakindentopt=
setlocal bufhidden=
setlocal buflisted
setlocal buftype=
setlocal cindent
setlocal cinkeys=0{,0},!^F,o,O,0[,0]
setlocal cinoptions=L0,(0,Ws,J1,j1
setlocal cinwords=for,if,else,while,loop,impl,mod,unsafe,trait,struct,enum,fn,extern
set colorcolumn=80
setlocal colorcolumn=80
setlocal comments=s0:/*!,m:\ ,ex:*/,s1:/*,mb:*,ex:*/,:///,://!,://
setlocal commentstring=//%s
setlocal complete=.,w,b,u,t,i
setlocal concealcursor=
setlocal conceallevel=0
setlocal completefunc=youcompleteme#Complete
setlocal nocopyindent
setlocal cryptmethod=
setlocal nocursorbind
setlocal nocursorcolumn
setlocal nocursorline
setlocal define=
setlocal dictionary=
setlocal nodiff
setlocal equalprg=
setlocal errorformat=
setlocal expandtab
if &filetype != 'rust'
setlocal filetype=rust
endif
setlocal fixendofline
setlocal foldcolumn=0
setlocal foldenable
setlocal foldexpr=0
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldmarker={{{,}}}
setlocal foldmethod=manual
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldtext=foldtext()
setlocal formatexpr=
setlocal formatoptions=croqnlj
setlocal formatlistpat=^\\s*\\d\\+[\\]:.)}\\t\ ]\\s*
setlocal grepprg=
setlocal iminsert=0
setlocal imsearch=0
setlocal include=
setlocal includeexpr=substitute(v:fname,'::','/','g')
setlocal indentexpr=GetRustIndent(v:lnum)
setlocal indentkeys=0{,0},!^F,o,O,0[,0]
setlocal noinfercase
setlocal iskeyword=@,48-57,_,192-255
setlocal keywordprg=
setlocal nolinebreak
setlocal nolisp
setlocal lispwords=
setlocal nolist
setlocal makeprg=
setlocal matchpairs=(:),{:},[:],<:>
setlocal modeline
setlocal modifiable
setlocal nrformats=octal,hex
set number
setlocal number
setlocal numberwidth=4
setlocal omnifunc=RacerComplete
setlocal path=
setlocal nopreserveindent
setlocal nopreviewwindow
setlocal quoteescape=\\
setlocal noreadonly
setlocal norelativenumber
setlocal norightleft
setlocal rightleftcmd=search
setlocal noscrollbind
setlocal shiftwidth=4
setlocal noshortname
setlocal smartindent
setlocal softtabstop=4
setlocal nospell
setlocal spellcapcheck=[.?!]\\_[\\])'\"\	\ ]\\+
setlocal spellfile=
setlocal spelllang=en
setlocal statusline=
setlocal suffixesadd=.rs
setlocal swapfile
setlocal synmaxcol=3000
if &syntax != 'rust'
setlocal syntax=rust
endif
setlocal tabstop=4
setlocal tagcase=
setlocal tags=
setlocal textwidth=99
setlocal thesaurus=
setlocal noundofile
setlocal undolevels=-123456
setlocal nowinfixheight
setlocal nowinfixwidth
setlocal wrap
setlocal wrapmargin=0
silent! normal! zE
let s:l = 178 - ((15 * winheight(0) + 26) / 53)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
178
normal! 0
wincmd w
argglobal
enew
setlocal keymap=
setlocal noarabic
setlocal noautoindent
setlocal backupcopy=
setlocal balloonexpr=
setlocal nobinary
setlocal nobreakindent
setlocal breakindentopt=
setlocal bufhidden=wipe
setlocal buflisted
setlocal buftype=quickfix
setlocal nocindent
setlocal cinkeys=0{,0},0),:,0#,!^F,o,O,e
setlocal cinoptions=
setlocal cinwords=if,else,while,do,for,switch
set colorcolumn=80
setlocal colorcolumn=80
setlocal comments=s1:/*,mb:*,ex:*/,://,b:#,:%,:XCOMM,n:>,fb:-
setlocal commentstring=/*%s*/
setlocal complete=.,w,b,u,t,i
setlocal concealcursor=
setlocal conceallevel=0
setlocal completefunc=
setlocal nocopyindent
setlocal cryptmethod=
setlocal nocursorbind
setlocal nocursorcolumn
setlocal nocursorline
setlocal define=
setlocal dictionary=
setlocal nodiff
setlocal equalprg=
setlocal errorformat=
setlocal expandtab
if &filetype != 'qf'
setlocal filetype=qf
endif
setlocal fixendofline
setlocal foldcolumn=0
setlocal foldenable
setlocal foldexpr=0
setlocal foldignore=#
setlocal foldlevel=0
setlocal foldmarker={{{,}}}
setlocal foldmethod=manual
setlocal foldminlines=1
setlocal foldnestmax=20
setlocal foldtext=foldtext()
setlocal formatexpr=
setlocal formatoptions=tcq
setlocal formatlistpat=^\\s*\\d\\+[\\]:.)}\\t\ ]\\s*
setlocal grepprg=
setlocal iminsert=0
setlocal imsearch=0
setlocal include=
setlocal includeexpr=
setlocal indentexpr=
setlocal indentkeys=0{,0},:,0#,!^F,o,O,e
setlocal noinfercase
setlocal iskeyword=@,48-57,_,192-255
setlocal keywordprg=
setlocal nolinebreak
setlocal nolisp
setlocal lispwords=
setlocal nolist
setlocal makeprg=
setlocal matchpairs=(:),{:},[:],<:>
setlocal modeline
setlocal nomodifiable
setlocal nrformats=octal,hex
set number
setlocal number
setlocal numberwidth=4
setlocal omnifunc=
setlocal path=
setlocal nopreserveindent
setlocal nopreviewwindow
setlocal quoteescape=\\
setlocal noreadonly
setlocal norelativenumber
setlocal norightleft
setlocal rightleftcmd=search
setlocal noscrollbind
setlocal shiftwidth=4
setlocal noshortname
setlocal nosmartindent
setlocal softtabstop=0
setlocal nospell
setlocal spellcapcheck=[.?!]\\_[\\])'\"\	\ ]\\+
setlocal spellfile=
setlocal spelllang=en
setlocal statusline=%t%{exists('w:quickfix_title')?\ '\ '.w:quickfix_title\ :\ ''}\ %=%-15(%l,%c%V%)\ %P
setlocal suffixesadd=
setlocal noswapfile
setlocal synmaxcol=3000
if &syntax != 'qf'
setlocal syntax=qf
endif
setlocal tabstop=8
setlocal tagcase=
setlocal tags=
setlocal textwidth=0
setlocal thesaurus=
setlocal noundofile
setlocal undolevels=-123456
setlocal winfixheight
setlocal nowinfixwidth
setlocal wrap
setlocal wrapmargin=0
wincmd w
exe 'vert 1resize ' . ((&columns * 167 + 117) / 235)
exe '2resize ' . ((&lines * 53 + 31) / 62)
exe 'vert 2resize ' . ((&columns * 67 + 117) / 235)
exe '3resize ' . ((&lines * 6 + 31) / 62)
exe 'vert 3resize ' . ((&columns * 67 + 117) / 235)
tabnext 1
if exists('s:wipebuf')
  silent exe 'bwipe ' . s:wipebuf
endif
unlet! s:wipebuf
set winheight=1 winwidth=20 shortmess=filnxtToO
let s:sx = expand("<sfile>:p:r")."x.vim"
if file_readable(s:sx)
  exe "source " . fnameescape(s:sx)
endif
let &so = s:so_save | let &siso = s:siso_save
doautoall SessionLoadPost
unlet SessionLoad
" vim: set ft=vim :
