package com.example.controller;

import com.example.dto.UserCreateRequest;
import com.example.dto.UserResponse;
import com.example.dto.UserUpdateRequest;
import com.example.exception.UserNotFoundException;
import com.example.model.User;
import com.example.service.UserService;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.validation.annotation.Validated;
import org.springframework.web.bind.annotation.*;

import javax.validation.Valid;
import javax.validation.constraints.Min;
import javax.validation.constraints.Positive;
import java.util.List;
import java.util.stream.Collectors;

/**
 * RESTful API controller for User management
 * Demonstrates modern Spring Boot controller patterns with annotations
 */
@RestController
@RequestMapping("/api/v1/users")
@Validated
@CrossOrigin(origins = "*")
public class UserController {
    
    private final UserService userService;
    
    public UserController(UserService userService) {
        this.userService = userService;
    }
    
    @GetMapping
    public ResponseEntity<List<UserResponse>> getAllUsers() {
        List<User> users = userService.findAll();
        List<UserResponse> responses = users.stream()
                .map(this::mapToResponse)
                .collect(Collectors.toList());
        return ResponseEntity.ok(responses);
    }
    
    @GetMapping("/{id}")
    public ResponseEntity<UserResponse> getUserById(
            @PathVariable @Positive Long id) {
        User user = userService.findById(id)
                .orElseThrow(() -> new UserNotFoundException("User not found with id: " + id));
        return ResponseEntity.ok(mapToResponse(user));
    }
    
    @GetMapping("/email/{email}")
    public ResponseEntity<UserResponse> getUserByEmail(
            @PathVariable String email) {
        User user = userService.findByEmail(email)
                .orElseThrow(() -> new UserNotFoundException("User not found with email: " + email));
        return ResponseEntity.ok(mapToResponse(user));
    }
    
    @GetMapping("/search")
    public ResponseEntity<List<UserResponse>> searchUsersByUsername(
            @RequestParam String username) {
        List<User> users = userService.findByUsernameContaining(username);
        List<UserResponse> responses = users.stream()
                .map(this::mapToResponse)
                .collect(Collectors.toList());
        return ResponseEntity.ok(responses);
    }
    
    @GetMapping("/active")
    public ResponseEntity<List<UserResponse>> getActiveUsers() {
        List<User> users = userService.findActiveUsers();
        List<UserResponse> responses = users.stream()
                .map(this::mapToResponse)
                .collect(Collectors.toList());
        return ResponseEntity.ok(responses);
    }
    
    @PostMapping
    @ResponseStatus(HttpStatus.CREATED)
    public ResponseEntity<UserResponse> createUser(
            @Valid @RequestBody UserCreateRequest request) {
        User user = mapToEntity(request);
        User savedUser = userService.save(user);
        return ResponseEntity.status(HttpStatus.CREATED).body(mapToResponse(savedUser));
    }
    
    @PutMapping("/{id}")
    public ResponseEntity<UserResponse> updateUser(
            @PathVariable @Positive Long id,
            @Valid @RequestBody UserUpdateRequest request) {
        User user = userService.findById(id)
                .orElseThrow(() -> new UserNotFoundException("User not found with id: " + id));
        
        User updatedUser = userService.updateUser(user, request);
        return ResponseEntity.ok(mapToResponse(updatedUser));
    }
    
    @DeleteMapping("/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public ResponseEntity<Void> deleteUser(
            @PathVariable @Positive Long id) {
        if (!userService.existsById(id)) {
            throw new UserNotFoundException("User not found with id: " + id);
        }
        userService.deleteById(id);
        return ResponseEntity.noContent().build();
    }
    
    @GetMapping("/count")
    public ResponseEntity<Long> getUserCount() {
        return ResponseEntity.ok(userService.count());
    }
    
    @GetMapping("/admin/{id}")
    @PreAuthorize("hasRole('ADMIN')")
    public ResponseEntity<UserResponse> getUserAdminView(
            @PathVariable @Positive Long id) {
        User user = userService.findById(id)
                .orElseThrow(() -> new UserNotFoundException("User not found with id: " + id));
        return ResponseEntity.ok(mapToResponse(user));
    }
    
    private UserResponse mapToResponse(User user) {
        return new UserResponse(
                user.getId(),
                user.getUsername(),
                user.getEmail(),
                user.getFirstName(),
                user.getLastName(),
                user.getRole(),
                user.isActive(),
                user.getCreatedAt(),
                user.getUpdatedAt()
        );
    }
    
    private User mapToEntity(UserCreateRequest request) {
        User user = new User();
        user.setUsername(request.getUsername());
        user.setEmail(request.getEmail());
        user.setFirstName(request.getFirstName());
        user.setLastName(request.getLastName());
        user.setPassword(request.getPassword());
        return user;
    }
}