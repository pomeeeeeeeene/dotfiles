STOW ?= stow
HOME_DIR ?= $(HOME)

.PHONY: stow unstow stow-kak unstow-kak

stow:
	$(STOW) yazi bin tmux ghostty
	$(MAKE) stow-kak

unstow:
	$(STOW) -D yazi bin tmux ghostty
	$(MAKE) unstow-kak

stow-kak:
	install -d $(HOME_DIR)/.config/kak
	$(STOW) -t $(HOME_DIR)/.config/kak kak

unstow-kak:
	$(STOW) -D -t $(HOME_DIR)/.config/kak kak
