package com.example.dto;

import com.example.model.User;
import java.time.LocalDateTime;

/**
 * Response DTO for user data
 */
public class UserResponse {
    private Long id;
    private String username;
    private String email;
    private String firstName;
    private String lastName;
    private User.UserRole role;
    private boolean active;
    private LocalDateTime createdAt;
    private LocalDateTime updatedAt;
    
    public UserResponse(Long id, String username, String email, String firstName, 
                       String lastName, User.UserRole role, boolean active, 
                       LocalDateTime createdAt, LocalDateTime updatedAt) {
        this.id = id;
        this.username = username;
        this.email = email;
        this.firstName = firstName;
        this.lastName = lastName;
        this.role = role;
        this.active = active;
        this.createdAt = createdAt;
        this.updatedAt = updatedAt;
    }
    
    // Getters only for response DTO
    public Long getId() { return id; }
    public String getUsername() { return username; }
    public String getEmail() { return email; }
    public String getFirstName() { return firstName; }
    public String getLastName() { return lastName; }
    public User.UserRole getRole() { return role; }
    public boolean isActive() { return active; }
    public LocalDateTime getCreatedAt() { return createdAt; }
    public LocalDateTime getUpdatedAt() { return updatedAt; }
    
    public String getFullName() {
        return (firstName != null ? firstName : "") + 
               (lastName != null ? " " + lastName : "");
    }
}