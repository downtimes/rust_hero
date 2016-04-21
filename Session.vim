let SessionLoad = 1
if &cp | set nocp | endif
let s:so_save = &so | let s:siso_save = &siso | set so=0 siso=0
let v:this_session=expand("<sfile>:p")
silent only
cd D:\WinWorkspace\rust_hero
if expand('%') == '' && !&modified && line('$') <= 1 && getline(1) == ''
  let s:wipebuf = bufnr('%')
endif
set shortmess=aoO
badd +9 src\main.rs
badd +167 src\common\mod.rs
badd +0 src\ffi\direct_sound.rs
badd +0 src\ffi\linux.rs
badd +667 src\ffi\mod.rs
badd +0 src\ffi\sdl.rs
badd +0 src\game\entity.rs
badd +0 src\game\graphics.rs
badd +0 src\game\math.rs
badd +0 src\game\memory.rs
badd +22 src\game\mod.rs
badd +0 src\game\random.rs
badd +0 src\game\simulation.rs
badd +0 src\game\world.rs
badd +0 src\libgame.rs
badd +0 src\linux.rs
badd +1405 src\win32.rs
args src\common\mod.rs src\ffi\direct_sound.rs src\ffi\linux.rs src\ffi\mod.rs src\ffi\sdl.rs src\game\entity.rs src\game\graphics.rs src\game\math.rs src\game\memory.rs src\game\mod.rs src\game\random.rs src\game\simulation.rs src\game\world.rs src\libgame.rs src\linux.rs src\main.rs src\win32.rs
edit src\game\simulation.rs
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
exe 'vert 1resize ' . ((&columns * 109 + 117) / 235)
exe '2resize ' . ((&lines * 47 + 30) / 60)
exe 'vert 2resize ' . ((&columns * 125 + 117) / 235)
exe '3resize ' . ((&lines * 10 + 30) / 60)
exe 'vert 3resize ' . ((&columns * 125 + 117) / 235)
argglobal
edit src\game\simulation.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let s:l = 43 - ((42 * winheight(0) + 29) / 58)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
43
normal! 037|
wincmd w
argglobal
edit src\common\mod.rs
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
silent! normal! zE
let s:l = 178 - ((43 * winheight(0) + 23) / 47)
if s:l < 1 | let s:l = 1 | endif
exe s:l
normal! zt
178
normal! 0
wincmd w
argglobal
enew
setlocal fdm=manual
setlocal fde=0
setlocal fmr={{{,}}}
setlocal fdi=#
setlocal fdl=0
setlocal fml=1
setlocal fdn=20
setlocal fen
wincmd w
exe 'vert 1resize ' . ((&columns * 109 + 117) / 235)
exe '2resize ' . ((&lines * 47 + 30) / 60)
exe 'vert 2resize ' . ((&columns * 125 + 117) / 235)
exe '3resize ' . ((&lines * 10 + 30) / 60)
exe 'vert 3resize ' . ((&columns * 125 + 117) / 235)
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
