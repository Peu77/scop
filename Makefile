NAME := scop
CARGO := cargo
TARGET := target/release/$(NAME)
SOURCES := $(shell find src -type f)

.PHONY: all run debug test clean fclean re

all: $(NAME)

$(NAME): $(SOURCES) Cargo.toml Cargo.lock
	$(CARGO) build --release
	cp $(TARGET) $(NAME)

run: $(NAME)
	./$(NAME) assets/cube.obj

debug:
	$(CARGO) build

test:
	$(CARGO) test

clean:
	$(CARGO) clean

fclean: clean
	rm -f $(NAME)

re: fclean all
