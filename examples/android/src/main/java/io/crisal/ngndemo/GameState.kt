package io.crisal.ngndemo

// Pretty inspired by:
//   https://github.com/gabrielecirulli/2048/blob/478b6ec346e3787f589e4af751378d06ded4cbbc/js/game_manager.js

import kotlin.random.Random

enum class GameState {
    NotYetStarted, MyTurn, OtherTurn, Lost, Won,
}

enum class MoveDirection {
    Up, Down, Left, Right,
}

data class Point(val x: Int, val y: Int)

data class Tile(var value: Int, var pos: Point)

data class FurthestPosition(val pos: Point, val next: Point)

class GameBoard {
    val size = 4
    var turn: GameState = GameState.NotYetStarted
        private set

    val board: Array<Array<Tile?>> = Array(size) { Array(size) { null } }
        private set

    val tiles = arrayListOf<Tile>()

    fun availableCells(): ArrayList<Point> {
        val result = arrayListOf<Point>()
        for (x in 0..<size) {
            for (y in 0..<size) {
                if (board[x][y] == null) {
                    result.add(Point(x, y))
                }
            }
        }
        return result
    }

    fun score(): Int {
        var score = 0
        for (tile in tiles) {
            score += tile.value
        }
        return score
    }

    fun tileAt(pos: Point): Tile? {
        if (pos.x < size && pos.y < size) {
            return board[pos.x][pos.y]
        }
        return null
    }

    fun cellAvailable(pos: Point): Boolean {
        return tileAt(pos) == null
    }

    fun findFurthestPosition(cell: Point, direction: Point): FurthestPosition {
        var cell = cell
        var previous: Point

        // Progress towards the vector direction until an obstacle is found
        do {
            previous = cell
            cell = Point(previous.x + direction.x, previous.y + direction.y)
        } while (cellAvailable(cell))

        return FurthestPosition(previous, cell)
    }

    private fun addTile(value: Int, pos: Point): Tile {
        assert(board[pos.x][pos.y] == null) { "Tile already placed" }
        val tile = Tile(value, pos)
        board[pos.x][pos.y] = tile
        tiles.add(tile)
        return tile
    }

    private fun removeTile(tile: Tile) {
        assert(tiles.contains(tile))
        assert(tileAt(tile.pos) == tile)
        board[tile.pos.x][tile.pos.y] = null
        tiles.remove(tile)
    }

    private fun moveTile(tile: Tile, pos: Point) {
        assert(tiles.contains(tile))
        assert(tileAt(tile.pos) == tile)
        board[tile.pos.x][tile.pos.y] = null
        tile.pos = pos
        board[pos.x][pos.y] = tile
    }

    fun tryAddRandomTile(): Tile? {
        val value = if (Random.nextFloat() < .9) {
            2
        } else {
            4
        }
        return tryAddNewTile(value)
    }

    fun start() {
        if (turn != GameState.NotYetStarted) {
            throw Error("Game is already started")
        }
        tryAddRandomTile()
        turn = GameState.MyTurn
    }

    fun startWithTile(tile: Tile) {
        if (turn != GameState.NotYetStarted) {
            throw Error("Game is already started")
        }
        addTile(tile.value, tile.pos)
        turn = GameState.MyTurn
    }

    fun tryAddNewTile(value: Int): Tile? {
        val cells = availableCells()
        if (cells.isEmpty()) {
            turn = GameState.Lost
            return null
        }
        val pos = cells[Random.nextInt(until = cells.size)]
        return addTile(value, pos)
    }

    fun tryMove(direction: MoveDirection): Tile? {
        if (turn != GameState.MyTurn || turn != GameState.OtherTurn) {
            return null
        }

        val (vector, xRange, yRange) = when (direction) {
            MoveDirection.Up -> Triple(Point(0, -1), 0 until size, 0 until size)
            MoveDirection.Down -> Triple(Point(0, 1), 0 until size, size - 1 downTo 0)
            MoveDirection.Left -> Triple(Point(-1, 0), 0 until size, 0 until size)
            MoveDirection.Right -> Triple(Point(1, 0), size - 1 downTo 0, 0 until size)
        }

        val merged = hashSetOf<Point>()
        var moved = false
        for (x in xRange) {
            for (y in yRange) {
                val pos = Point(x, y)
                val tile = tileAt(pos)
                if (tile == null) {
                    continue
                }
                val furthest = findFurthestPosition(pos, vector)
                val next = tileAt(furthest.next)
                if (next != null && next.value == tile.value && !merged.add(furthest.next)) {
                    next.value *= 2
                    // Remove the tile we merged into `next`.
                    removeTile(tile)
                    moved = true
                    if (next.value == 2048) {
                        turn = GameState.Won
                    }
                } else {
                    if (pos != furthest.pos) {
                        moveTile(tile, furthest.pos)
                        moved = true // The tile moved from its original cell!
                    }
                }
            }
        }
        if (!moved) {
            return null
        }
        val tile = tryAddRandomTile()
        turn = if (tile == null) {
            GameState.Lost
        } else if (turn == GameState.MyTurn) {
            GameState.OtherTurn
        } else {
            GameState.MyTurn
        }
        return tile
    }
}
