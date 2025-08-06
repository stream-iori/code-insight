package com.example.dto;

import javax.validation.constraints.*;
import com.example.model.User;

/**
 * Request DTO for updating a user
 */
public class UserUpdateRequest {
    
    @NotBlank(message = "Username is required")
    @Size(min = 3, max = 50, message = "Username must be between 3 and 50 characters")
    private String username;
    
    @Email(message = "Email must be valid")
    private String email;
    
    @Size(max = 100, message = "First name cannot exceed 100 characters")
    private String firstName;
    
    @Size(max = 100, message = "Last name cannot exceed 100 characters")
    private String lastName;
    
    private User.UserRole role;
    private boolean active;
    
    public String getUsername() { return username; }
    public void setUsername(String username) { this.username = username; }
    
    public String getEmail() { return email; }
    public void setEmail(String email) { this.email = email; }
    
    public String getFirstName() { return firstName; }
    public void setFirstName(String firstName) { this.firstName = firstName; }
    
    public String getLastName() { return lastName; }
    public void setLastName(String lastName) { this.lastName = lastName; }
    
    public User.UserRole getRole() { return role; }
    public void setRole(User.UserRole role) { this.role = role; }
    
    public boolean isActive() { return active; }
    public void setActive(boolean active) { this.active = active; }
}